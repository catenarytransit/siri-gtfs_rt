use csv::Writer;
use geo::GeodesicLength;
use geo_types::geometry::LineString;
use gtfs_structures;
use polyline;
use reqwest::{header, Client as ReqwestClient};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::fs::File;
use std::io::Write;
mod timeutil;
use timeutil::string_h_m_to_u32;

use geojson::GeoJson;

#[derive(Deserialize, Debug, Serialize)]
struct ScheduleManualInput {
    name: String,
    routeorder: Vec<String>,
    timed: Vec<String>,
    files: Vec<ScheduleFiles>,
    extrasegments: Option<Vec<Vec<String>>>,
    overrideshape: Option<String>,
}

#[derive(Copy, Clone)]
struct ComparisonOfSegments {
    distance: f64,
    index: usize,
}

#[derive(Deserialize, Debug, Serialize)]
struct ScheduleFiles {
    name: String,
    //true if monday-thursday, false if friday only
    monthurs: bool,
}

#[derive(Deserialize, Debug, Serialize)]
struct TranslocAgencies {
    rate_limit: f32,
    expires_in: f32,
    api_latest_version: String,
    generated_on: String,
    data: Vec<TranslocAgency>,
    api_version: String,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct TranslocPos {
    lat: f32,
    lng: f32,
}

#[derive(Deserialize, Debug, Serialize)]
struct TranslocAgency {
    long_name: String,
    language: String,
    position: TranslocPos,
    short_name: String,
    name: String,
    phone: Option<String>,
    url: String,
    timezone: String,
    bounding_box: Vec<TranslocPos>,
    agency_id: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct TranslocSegments {
    data: BTreeMap<String, String>,
    api_version: String,
    rate_limit: f32,
    expires_in: f32,
    api_latest_version: String,
    generated_on: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct TranslocRoute {
    description: String,
    long_name: String,
    segments: Vec<Vec<String>>,
    short_name: String,
    //does not have #
    color: String,
    text_color: String,
    is_active: bool,
    route_id: String,
    agency_id: i32,
    url: String,
    #[serde(rename(deserialize = "type"))]
    route_type: String,
    is_hidden: bool,
    stops: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct TranslocRoutes {
    data: BTreeMap<String, Vec<TranslocRoute>>,
    api_version: String,
    rate_limit: f32,
    expires_in: f32,
    api_latest_version: String,
    generated_on: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TranslocStop {
    code: String,
    description: String,
    url: String,
    parent_station_id: Option<String>,
    agency_ids: Vec<String>,
    station_id: Option<String>,
    location_type: String,
    location: TranslocPos,
    stop_id: String,
    routes: Vec<String>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct TranslocStops {
    data: Vec<TranslocStop>,
    api_version: String,
    rate_limit: f32,
    expires_in: f32,
    api_latest_version: String,
    generated_on: String,
}

#[derive(Debug, Clone)]
struct Segmentinfo {
    segment_id: String,
    data: Vec<geo_types::geometry::Coord>,
    length: f64,
}

fn hextorgb(x: String) -> rgb::RGB<u8> {
    let numberarray: Vec<char> = x.chars().collect();

    let mut rgbarray = vec![];

    for i in 0..3 {
        let mut hex = numberarray[i * 2].to_string();
        hex.push_str(numberarray[i * 2 + 1].to_string().as_str());

        rgbarray.push(u8::from_str_radix(&hex, 16).unwrap());
    }

    return rgb::RGB8 {
        r: rgbarray[0],
        g: rgbarray[1],
        b: rgbarray[2],
    };
}

#[tokio::main]
async fn main() {
    let blockedfromboardings = vec!["201"];

    let manualschedule: Vec<ScheduleManualInput> =
        serde_json::from_str(fs::read_to_string("route-sup.json").unwrap().as_str()).unwrap();

    println!("{:?}", &manualschedule);

    let manualhashmap: HashMap<String, ScheduleManualInput> =
        HashMap::from_iter(manualschedule.into_iter().map(|x| (x.name.clone(), x)));

    let agenciesjson =
        fs::read_to_string("staticfiles/agencies.json").expect("Unable to read file");
    let agencies: TranslocAgencies = serde_json::from_str(&agenciesjson).unwrap();

    let gtfs_agencies = agencies
        .data
        .iter()
        .map(|agency| gtfs_structures::Agency {
            id: Some(agency.agency_id.clone()),
            name: agency.name.clone(),
            url: agency.url.clone(),
            timezone: String::from("America/Los_Angeles"),
            lang: Some(agency.language.clone()),
            phone: agency.phone.clone(),
            fare_url: None,
            email: None,
        })
        .collect::<Vec<gtfs_structures::Agency>>();

    println!("gtfs_agencies: {:?}", gtfs_agencies);

    let mut wtr = Writer::from_writer(vec![]);
    for agency in gtfs_agencies {
        wtr.serialize(agency).unwrap();
    }

    let gtfs_agencies_csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    println!("gtfs_agencies_csv: {:?}", gtfs_agencies_csv);

    let mut file = File::create("anteater_gtfs/agency.txt").unwrap();
    file.write_all(gtfs_agencies_csv.as_bytes()).unwrap();

    //time to decompile the segments :-(

    let segmentsjson =
        fs::read_to_string("staticfiles/segments.json").expect("Unable to read file");

    let segments: TranslocSegments = serde_json::from_str(&segmentsjson).unwrap();

    let routesjson = fs::read_to_string("staticfiles/routes.json").expect("Unable to read file");

    let routes: TranslocRoutes = serde_json::from_str(&routesjson).unwrap();

    let stopsjson = fs::read_to_string("staticfiles/stops.json").expect("Unable to read file");

    let stops: TranslocStops = serde_json::from_str(&stopsjson).unwrap();

    let mut segments_map: HashMap<String, LineString<f64>> = HashMap::new();

    let stopcode_to_stopid: HashMap<String, String> = HashMap::from_iter(
        stops
            .data
            .clone()
            .into_iter()
            .map(|stop: TranslocStop| (stop.code.clone(), stop.stop_id.clone())),
    );

    let mut stopswriter = Writer::from_writer(vec![]);

    let mut stoptimeswriter = Writer::from_writer(vec![]);
    let mut tripswriter = Writer::from_writer(vec![]);

    let stopshashmap: HashMap<String, gtfs_structures::Stop> =
        HashMap::from_iter(stops.data.clone().into_iter().map(|stop: TranslocStop| {
            (
                stop.stop_id.clone(),
                gtfs_structures::Stop {
                    id: stop.stop_id.clone(),
                    code: Some(stop.code.clone()),
                    name: stop.name.clone(),
                    description: stop.description.clone(),
                    location_type: gtfs_structures::LocationType::StopPoint,
                    parent_station: None,
                    zone_id: None,
                    url: None,
                    timezone: Some(String::from("America/Los_Angeles")),
                    latitude: Some(stop.location.lat.into()),
                    longitude: Some(stop.location.lng.into()),
                    wheelchair_boarding: gtfs_structures::Availability::Available,
                    level_id: None,
                    platform_code: None,
                    transfers: vec![],
                    pathways: vec![],
                },
            )
        }));

    //STOPS
    for stop in stops.data.iter() {
        stopswriter
            .serialize(gtfs_structures::Stop {
                id: stop.stop_id.clone(),
                code: Some(stop.code.clone()),
                name: stop.name.clone().replace(" Stop ", ""),
                description: stop.description.clone(),
                location_type: gtfs_structures::LocationType::StopPoint,
                parent_station: None,
                zone_id: None,
                url: None,
                timezone: Some(String::from("America/Los_Angeles")),
                latitude: Some(stop.location.lat.into()),
                longitude: Some(stop.location.lng.into()),
                wheelchair_boarding: gtfs_structures::Availability::Available,
                level_id: None,
                platform_code: None,
                transfers: vec![],
                pathways: vec![],
            })
            .unwrap();
    }

    let stops_csv = String::from_utf8(stopswriter.into_inner().unwrap()).unwrap();
    let mut stopsfile = File::create("anteater_gtfs/stops.txt").unwrap();

    stopsfile.write_all(stops_csv.as_bytes()).unwrap();

    let mut segments_polyline_debug = Writer::from_writer(vec![]);

    #[derive(Serialize, Clone)]
    struct debugpolyline {
        lat: f64,
        lng: f64,
        segment_id: String,
        number: usize,
    }

    for (segment_id, segment_data) in segments.data.iter() {
        //println!("{}, {}",segment_id, segment_data)

        let segment_polyline = polyline::decode_polyline(segment_data, 5).unwrap();

        segments_map.insert(segment_id.clone(), segment_polyline.clone());

        segment_polyline
            .coords()
            .enumerate()
            .for_each(|(i, coord)| {
                segments_polyline_debug
                    .serialize(debugpolyline {
                        lat: coord.y,
                        lng: coord.x,
                        segment_id: segment_id.clone(),
                        number: i,
                    })
                    .unwrap()
            });
    }

    let segments_polyline_debug_csv =
        String::from_utf8(segments_polyline_debug.into_inner().unwrap()).unwrap();
    let mut segments_polyline_debug_file = File::create("segments_polyline_debug.csv").unwrap();

    segments_polyline_debug_file
        .write_all(segments_polyline_debug_csv.as_bytes())
        .unwrap();

    println!("segments_map: {:?}", segments_map);

    let mut shapeswriter = Writer::from_writer(vec![]);

    let mut routeswriter = Writer::from_writer(vec![]);

    for (agency_id, routes_array) in routes.data.iter() {
        for route in routes_array {
            // println!("Route: {:?}", route);

            //make the schedule here

            let data_to_use = manualhashmap.get(&route.short_name).unwrap();

            println!("data_to_use: {:?}", data_to_use);

            let mut vec_of_trips: Vec<gtfs_structures::RawTrip> = vec![];
            let mut vec_of_stop_times: Vec<gtfs_structures::RawStopTime> = vec![];

            routeswriter
                .serialize(gtfs_structures::Route {
                    id: route.route_id.clone(),
                    agency_id: Some(String::from(agency_id)),
                    short_name: route.short_name.clone(),
                    long_name: format!("{} Line - Anteater Express", route.short_name.clone()),
                    desc: Some(route.description.clone()),
                    route_type: gtfs_structures::RouteType::Bus,
                    url: Some(format!(
                        "https://shuttle.uci.edu/routes/{}-line",
                        route.short_name.clone().as_str().to_lowercase()
                    )),
                    color: hextorgb(route.color.clone()),
                    text_color: hextorgb(route.text_color.clone()),
                    order: None,
                    continuous_pickup: gtfs_structures::ContinuousPickupDropOff::NotAvailable,
                    continuous_drop_off: gtfs_structures::ContinuousPickupDropOff::NotAvailable,
                })
                .unwrap();

            let mut arrayofsegments: Vec<Segmentinfo> = vec![];

            let mut sourcedata = route.segments.clone();

            if data_to_use.extrasegments.is_some() {
                for x in data_to_use.extrasegments.clone().unwrap() {
                    println!("Joining extra {:?} to route {}", x, route.route_id);
                    sourcedata.push(x.clone());
                }
            }

            for segment_part in &sourcedata {
                let segment_id = segment_part[0].clone();
                //can be "forward" or "backward"
                let segment_direction = segment_part[1].clone();

                let mut segment = segments_map.get(&segment_id).unwrap().clone().into_inner();

                if segment_direction == "backward" {
                    println!(
                        "reversing segment {} of route {}",
                        segment_id, route.route_id
                    );
                    segment.reverse();
                }

                let dataofthisseg = segment
                    .clone()
                    .into_iter()
                    .collect::<Vec<geo_types::geometry::Coord>>();
                //Lazy o(n^2) algo
                arrayofsegments.push(Segmentinfo {
                    segment_id: segment_id.clone(),
                    data: dataofthisseg.clone(),
                    length: geo::geometry::LineString::from_iter(
                        dataofthisseg
                            .iter()
                            .map(|x| geo::geometry::Point::new(x.x, x.y)),
                    )
                    .geodesic_length(),
                });
            }

            //sort array
            arrayofsegments
                .sort_by(|a, b| bool::cmp(&(a.length < b.length), &(b.length < a.length)));

            println!(
                "segments {:?}",
                arrayofsegments
                    .iter()
                    .map(|x| x.length)
                    .collect::<Vec<f64>>()
            );

            let mut segmentordered: LineString<f64> = LineString(vec![]);

            segmentordered = segmentordered
                .into_iter()
                .chain(arrayofsegments[0].data.clone().into_iter())
                .collect::<LineString<f64>>();

            println!("segmentordered {:?}", segmentordered);

            arrayofsegments.remove(0);

            while arrayofsegments.len() > 0 {
                let mut closest_end_to_my_start: Option<ComparisonOfSegments> = None;
                let mut closest_start_to_my_end: Option<ComparisonOfSegments> = None;

                let coordsofmyself: Vec<geo_types::Coord> = segmentordered
                    .clone()
                    .into_iter()
                    .map(|coord| coord)
                    .collect::<Vec<geo_types::Coord>>();

                let my_start = coordsofmyself[0].clone();
                let my_end = segmentordered[coordsofmyself.len() - 1].clone();

                // println!("my start {:?}", my_start);

                for (index, segment) in arrayofsegments.iter().enumerate() {
                    let start_partner = segment.data[0].clone();
                    let end_partner = segment.data[segment.data.len() - 1].clone();

                    //println!("their end {:?}", end_partner);
                    let my_start_to_their_end_distance =
                        vincenty_core::distance_from_coords(&my_start, &end_partner).unwrap();

                    let my_end_to_their_start_distance =
                        vincenty_core::distance_from_coords(&my_end, &start_partner).unwrap();

                    if (closest_end_to_my_start.is_none()
                        || my_start_to_their_end_distance
                            < closest_end_to_my_start.unwrap().distance)
                    {
                        closest_end_to_my_start = Some(ComparisonOfSegments {
                            distance: my_start_to_their_end_distance,
                            index: index,
                        });
                    }

                    if (closest_start_to_my_end.is_none()
                        || my_end_to_their_start_distance
                            < closest_start_to_my_end.unwrap().distance)
                    {
                        closest_start_to_my_end = Some(ComparisonOfSegments {
                            distance: my_end_to_their_start_distance,
                            index: index,
                        });
                    }
                }

                let mut index_to_remove: Option<usize> = None;

                if closest_end_to_my_start.unwrap().distance
                    < closest_start_to_my_end.unwrap().distance
                {
                    //join partner + me
                    segmentordered = arrayofsegments[closest_end_to_my_start.unwrap().index]
                        .data
                        .clone()
                        .into_iter()
                        .chain(segmentordered.into_iter())
                        .collect::<LineString<f64>>();

                    //drop the segment
                    index_to_remove = Some(closest_end_to_my_start.unwrap().index);
                } else {
                    //join me + partner

                    segmentordered = segmentordered
                        .into_iter()
                        .chain(
                            arrayofsegments[closest_start_to_my_end.unwrap().index]
                                .data
                                .clone()
                                .into_iter(),
                        )
                        .collect::<LineString<f64>>();

                    //drop the segment
                    index_to_remove = Some(closest_start_to_my_end.unwrap().index);
                }

                if index_to_remove.is_some() {
                    arrayofsegments.remove(index_to_remove.unwrap());
                }
            }

            //now they have to be seperated and put into the shapes list

            if data_to_use.overrideshape.is_some() {
                let file = File::open(data_to_use.overrideshape.clone().unwrap()).unwrap();

                let geojson = GeoJson::from_reader(file).unwrap();

                let mut linestring = match geojson {
                    GeoJson::FeatureCollection(feature_collection) => {
                        let mut linestring = None;

                        for feature in feature_collection.features {
                            if let Some(geojson_geometry) = feature.geometry {
                                match geojson_geometry.value {
                                    geojson::Value::LineString(line_string) => {
                                        linestring = Some(line_string);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        linestring
                    }
                    _ => None,
                };

                if linestring.is_some() {
                    segmentordered = LineString::from(
                        linestring
                            .unwrap()
                            .into_iter()
                            .map(|x| geo_types::Point::new(x[0], x[1]))
                            .collect::<Vec<geo_types::Point<f64>>>(),
                    );
                }
            }

            let this_routes_shape_id = format!("{}shape", route.route_id);

            let mut seqcount = 0;

            for point in segmentordered.into_iter() {
                shapeswriter
                    .serialize(gtfs_structures::Shape {
                        id: this_routes_shape_id.clone(),
                        latitude: point.y,
                        longitude: point.x,
                        sequence: seqcount,
                        dist_traveled: None,
                    })
                    .unwrap();

                seqcount = seqcount + 1;
            }

            for file in data_to_use.files.iter() {
                // let scheduletimes = fs::read_to_string(format!("schedules/{}", file.name).as_str()).unwrap().as_str();

                let mut rdr = csv::Reader::from_path(format!("schedules/{}", file.name)).unwrap();
                let vecofrows = rdr
                    .records()
                    .map(|x| x.unwrap())
                    .collect::<Vec<csv::StringRecord>>();

                println!("vecofrows: {:?}", vecofrows);

                let headers = rdr.headers().unwrap();

                if (headers.get(0).unwrap() == "repeat_interval"
                    && headers.get(1).unwrap() == "repeat_number"
                    && headers.len() >= 3)
                {
                    //convert header to vec
                    let mut headervec = vec![];

                    for i in 2..headers.len() {
                        headervec.push(headers.get(i).unwrap().to_string());
                    }

                    struct Stoptimepre {
                        stop_id: String,
                        stop_code: String,
                        timed: bool,
                        arrivals: Option<u32>,
                        departures: Option<u32>,
                        enabled: bool,
                        boardingsallowed: bool,
                    }

                    let mut tripnumber = 1;

                    //for each row
                    for row in vecofrows {
                        let mut scheduleforthistime: Vec<String> = vec![];

                        for i in 2..row.len() {
                            scheduleforthistime.push(row.get(i).unwrap().to_string());
                        }

                        //split into array of f32 based on /, minutes to seconds
                        let repeatinterval = row
                            .get(0)
                            .unwrap_or_else(|| "0")
                            .to_string()
                            .split("/")
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|x| x.parse::<f32>().unwrap_or_else(|x| 0.0) * 60.0)
                            .collect::<Vec<f32>>();

                        println!("next bus in {:?}s", repeatinterval);

                        let repeatnumberoftimes = row
                            .get(1)
                            .unwrap_or_else(|| "0")
                            .to_string()
                            .parse::<u32>()
                            .unwrap_or_else(|x| 1);

                        println!("repeat {:?} times", repeatnumberoftimes);

                        /*
                        rowvec.push(Stoptimepre {
                            stop_id: stopcode_to_stopid
                                .get(&routelooppoint.clone())
                                .unwrap()
                                .clone(),
                            stop_code: routelooppoint.clone(),
                            timed: data_to_use.timed.contains(&routelooppoint.clone()),
                            arrivals: None,
                            departures: None,
                            enabled: true,
                        }); */

                        let firsttimedindex = scheduleforthistime
                            .iter()
                            .position(|x| x.contains(":"))
                            .unwrap();
                        let initialtime =
                            string_h_m_to_u32(scheduleforthistime[firsttimedindex].clone());
                        let mut offset = 0;

                        for eachtrip in 0..repeatnumberoftimes {
                            //inclusive, cancel everything from 0 until
                            let mut cancelindexpre: Option<usize> = None;
                            let mut canceldeparturesindex: Option<usize> = None;

                            let mut rowvec: Vec<Stoptimepre> = vec![];
                            //process each stop along this trip
                            let whichintervaltouse =
                                repeatinterval[eachtrip as usize % repeatinterval.len() as usize];

                            let mut stopnumber = 0;

                            for (routeordercounter, routelooppoint) in
                                data_to_use.routeorder.iter().enumerate()
                            {
                                let mut calcboardingsallowed = true;

                                if routeordercounter == data_to_use.routeorder.len() - 1 {
                                    calcboardingsallowed = false;
                                }

                                if blockedfromboardings.contains(&(routelooppoint.as_str())) {
                                    calcboardingsallowed = false;
                                }

                                rowvec.push(Stoptimepre {
                                    stop_id: stopcode_to_stopid
                                        .get(&routelooppoint.clone())
                                        .unwrap()
                                        .clone(),
                                    stop_code: routelooppoint.clone(),
                                    timed: data_to_use.timed.contains(&routelooppoint.clone()),
                                    arrivals: None,
                                    departures: None,
                                    enabled: true,
                                    boardingsallowed: calcboardingsallowed,
                                });

                                //use scheduleforthistime to get the initial times
                                let mut departuretime = None;

                                if headervec.contains(&routelooppoint.clone()) {
                                    let index = headervec
                                        .iter()
                                        .position(|r| r.as_str() == routelooppoint.clone().as_str())
                                        .unwrap();

                                    let stringofdeparturetime = scheduleforthistime[index].clone();

                                    if cleanupstring(stringofdeparturetime.clone()).as_str() == "" {
                                        departuretime = None;
                                    } else {
                                        if stringofdeparturetime.contains(":") {
                                            departuretime = Some(
                                                string_h_m_to_u32(stringofdeparturetime.clone())
                                                    + offset,
                                            );
                                        }
                                    }

                                    if stringofdeparturetime.contains("*") {
                                        cancelindexpre = Some(stopnumber - 1);
                                        println!(
                                            "Cancelled all service 0 to {}",
                                            cancelindexpre.unwrap()
                                        )
                                    }

                                    if stringofdeparturetime.contains("$") {
                                        canceldeparturesindex = Some(stopnumber + 1);

                                        println!("Cancelled all boardings with at least {} stopnumber on line {} at time {}:{}", canceldeparturesindex.unwrap(), route.short_name, 
                                    
                                        (initialtime + offset) / 3600, ((initialtime + offset )% 3600) / 60
                                    )
                                    }
                                }

                                //cleanup the loop point by disabling it
                                if cancelindexpre.is_some() {
                                    if stopnumber <= cancelindexpre.unwrap() {
                                        rowvec[stopnumber].enabled = false;
                                    }
                                }

                                if canceldeparturesindex.is_some() {
                                    if stopnumber >= canceldeparturesindex.unwrap() {
                                        rowvec[stopnumber].boardingsallowed = false;
                                    }
                                }

                                rowvec[stopnumber].departures = departuretime;

                                stopnumber = stopnumber + 1;
                            }

                            offset = offset + whichintervaltouse as u32;
                            tripnumber = tripnumber + 1;

                            //write the data to the csv

                            //get monthurs

                            let schedulename = match file.monthurs {
                                true => "monthurs",
                                false => "fri",
                            };

                            let trip_id =
                                format!("{}-{}-{}", route.short_name, tripnumber, schedulename);

                            let rawtripgtfs = gtfs_structures::RawTrip {
                                id: trip_id.clone(),
                                service_id: schedulename.to_string(),
                                route_id: route.route_id.clone(),
                                direction_id: Some(gtfs_structures::DirectionType::Outbound),
                                block_id: None,
                                trip_headsign: None,
                                trip_short_name: None,
                                shape_id: Some(this_routes_shape_id.clone()),
                                bikes_allowed: gtfs_structures::BikesAllowedType::AtLeastOneBike,
                                wheelchair_accessible: gtfs_structures::Availability::Available,
                            };

                            tripswriter.serialize(rawtripgtfs).unwrap();

                            let mut stopcounterfinal = 0;
                            for stoptimefinal in rowvec.iter() {
                                if stoptimefinal.enabled {
                                    let rawstoptimegtfs = gtfs_structures::RawStopTime {
                                        trip_id: trip_id.clone(),
                                        arrival_time: stoptimefinal.arrivals,
                                        departure_time: stoptimefinal.departures,
                                        stop_id: stoptimefinal.stop_id.clone(),
                                        stop_sequence: stopcounterfinal,
                                        stop_headsign: None,
                                        pickup_type: match stoptimefinal.boardingsallowed {
                                            true => gtfs_structures::PickupDropOffType::Regular,
                                            false => {
                                                gtfs_structures::PickupDropOffType::NotAvailable
                                            }
                                        },
                                        drop_off_type: gtfs_structures::PickupDropOffType::Regular,
                                        shape_dist_traveled: None,
                                        timepoint: match stoptimefinal.timed {
                                            true => gtfs_structures::TimepointType::Exact,
                                            false => gtfs_structures::TimepointType::Approximate,
                                        },
                                        continuous_pickup:
                                            gtfs_structures::ContinuousPickupDropOff::NotAvailable,
                                        continuous_drop_off:
                                            gtfs_structures::ContinuousPickupDropOff::NotAvailable,
                                    };

                                    stoptimeswriter.serialize(rawstoptimegtfs).unwrap();

                                    stopcounterfinal = stopcounterfinal + 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let shapes_csv = String::from_utf8(shapeswriter.into_inner().unwrap()).unwrap();
    let mut shapesfile = File::create("anteater_gtfs/shapes.txt").unwrap();

    shapesfile.write_all(shapes_csv.as_bytes()).unwrap();

    let routes_csv = String::from_utf8(routeswriter.into_inner().unwrap()).unwrap();
    let mut routesfile = File::create("anteater_gtfs/routes.txt").unwrap();

    routesfile.write_all(routes_csv.as_bytes()).unwrap();

    let trips_csv = String::from_utf8(tripswriter.into_inner().unwrap()).unwrap();
    let mut tripsfile = File::create("anteater_gtfs/trips.txt").unwrap();

    tripsfile.write_all(trips_csv.as_bytes()).unwrap();

    let stoptimes_csv = String::from_utf8(stoptimeswriter.into_inner().unwrap()).unwrap();
    let mut stoptimesfile = File::create("anteater_gtfs/stop_times.txt").unwrap();

    stoptimesfile.write_all(stoptimes_csv.as_bytes()).unwrap();

    let mut calendarwriter = Writer::from_writer(vec![]);

    calendarwriter
        .serialize(gtfs_structures::Calendar {
            id: String::from("monthurs"),
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: false,
            saturday: false,
            sunday: false,
            start_date: chrono::naive::NaiveDate::from_ymd_opt(2024, 04, 01).unwrap(),
            end_date: chrono::naive::NaiveDate::from_ymd_opt(2023, 06, 14).unwrap(),
        })
        .unwrap();

    calendarwriter
        .serialize(gtfs_structures::Calendar {
            id: String::from("fri"),
            monday: false,
            tuesday: false,
            wednesday: false,
            thursday: false,
            friday: true,
            saturday: false,
            sunday: false,
            start_date: chrono::naive::NaiveDate::from_ymd_opt(2024, 04, 01).unwrap(),
            end_date: chrono::naive::NaiveDate::from_ymd_opt(2024, 06, 14).unwrap(),
        })
        .unwrap();

    let calendar_csv = String::from_utf8(calendarwriter.into_inner().unwrap()).unwrap();
    let mut calendarfile = File::create("anteater_gtfs/calendar.txt").unwrap();

    calendarfile.write_all(calendar_csv.as_bytes()).unwrap();

    let mut calendardateswriter = Writer::from_writer(vec![]);

    let cancellations = [
        //Memorial day
        ((2024, 05, 27), "monthurs"),
    ];

    for cancel in cancellations {
        calendardateswriter
            .serialize(gtfs_structures::CalendarDate {
                service_id: String::from(cancel.1),
                date: chrono::naive::NaiveDate::from_ymd_opt(cancel.0 .0, cancel.0 .1, cancel.0 .2)
                    .unwrap(),
                exception_type: gtfs_structures::Exception::Deleted,
            })
            .unwrap();
    }

    //write now

    let calendardates_csv = String::from_utf8(calendardateswriter.into_inner().unwrap()).unwrap();
    let mut calendardatesfile = File::create("anteater_gtfs/calendar_dates.txt").unwrap();

    calendardatesfile
        .write_all(calendardates_csv.as_bytes())
        .unwrap();

    //now validate it

    let gtfs = gtfs_structures::GtfsReader::default().read("anteater_gtfs");

    match gtfs {
        Ok(gtfs) => {
            println!("Valid");
        }
        Err(e) => {
            println!("error: {:?}", e);
        }
    }
}

fn cleanupstring(x: String) -> String {
    return x
        .replace("!", "")
        .replace("#", "")
        .replace("*", "")
        .replace("$", "")
        .replace(" ", "");
}
