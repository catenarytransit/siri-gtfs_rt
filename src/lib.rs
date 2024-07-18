use chrono::{DateTime, FixedOffset, ParseResult};
use gtfs_rt::*;
use gtfs_structures::*;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use vehicle_position::CarriageDetails;

const API_KEY: &str = "UYEHABM01C9";

pub async fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>>
{
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}

fn parse_data(
    data: String,
) -> Result<VehicleMonitoringDelivery, Box<dyn std::error::Error + Send + Sync>> {
    let data: Siri = from_str(&data)?;
    Ok(data.service_delivery.vehicle_monitoring_delivery)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Siri {
    service_delivery: ServiceDelivery,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ServiceDelivery {
    vehicle_monitoring_delivery: VehicleMonitoringDelivery,
    response_timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleMonitoringDelivery {
    vehicle_activity: Option<Vec<VehicleActivity>>,
    valid_until: String,
    response_timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleActivity {
    monitored_vehicle_journey: MonitoredVehicleJourney,
    recorded_at_time: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct MonitoredVehicleJourney {
    extensions: Extensions,
    line_ref: LineRef,
    framed_vehicle_journey_ref: FramedVehicleJourneyRef,
    published_line_name: Vec<PublishedLineName>,
    direction_name: Vec<DirectionName>,
    origin_ref: OriginRef,
    destination_ref: DestinationRef,
    destination_name: Vec<DestinationName>,
    monitored: bool,
    vehicle_location: VehicleLocation,
    bearing: f32,
    progress_status: Vec<ProgressStatus>,
    course_of_journey_ref: CourseOfJourneyRef,
    vehicle_ref: VehicleRef,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Extensions {
    #[serde(rename = "lastGPSFix")]
    last_gps_fix: String, //time
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LineRef {
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FramedVehicleJourneyRef {
    data_frame_ref: DataFrameRef,
    dated_vehicle_journey_ref: String, //i32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PublishedLineName {
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DirectionName {
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OriginRef {
    value: String, //i32, origin station code
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DestinationRef {
    value: String, //i32, destination station code
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DestinationName {
    value: String, //destination station name
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VehicleLocation {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProgressStatus {
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CourseOfJourneyRef {
    value: String, //i32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VehicleRef {
    value: String, //u16
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DataFrameRef {
    value: String,
}

/**
 * Impl block for VehicleMonitoringDelivery, aka a set of vehicles from an api call.
 */
impl VehicleMonitoringDelivery {
    /**
     * Fetches the response timestamp.
     */
    fn get_response_timestamp(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.response_timestamp)
    }

    /**
     * Fetches the expiration timestamp.
     */
    fn get_valid_until(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.valid_until)
    }

    /**
     * Creates a feed message using data from each vehicle and the response timestamp.
     * only function out of these impl blocks that should be used.
     */
    fn get_feed_message(&self, gtfs: &Gtfs) -> Option<gtfs_rt::FeedMessage> {
        Some(FeedMessage {
            header: self.get_feed_header(),
            entity: self.get_feed_entities(gtfs),
        })
    }

    /**
     * Creates a feed header by converting the response timestamp into a unix
     * timestamp.
     */
    fn get_feed_header(&self) -> gtfs_rt::FeedHeader {
        FeedHeader {
            gtfs_realtime_version: String::from("2.0"),
            incrementality: Some(1),
            timestamp: Some(
                self.get_response_timestamp()
                    .unwrap_or_default()
                    .timestamp() as u64,
            ),
        }
    }

    /**
     * Creates a feed entity for each vehicle in the siri call and compiles them into
     * a vector.
     */
    fn get_feed_entities(&self, gtfs: &Gtfs) -> Vec<gtfs_rt::FeedEntity> {
        let vehicles = match &self.vehicle_activity {
            Some(x) => x,
            None => return Vec::new(),
        };
        let mut entities: Vec<FeedEntity> = Vec::new();

        // for every vehicle
        for vehicle in vehicles {
            match vehicle.get_feed_entity(gtfs) {
                // if vehicle creates a valid feed_entity
                Some(x) => entities.push(x),
                // else add nothing
                None => (),
            }
        }
        entities
    }
}

/**
 * Impl block for VehicleActivity, aka an individual vehicle from an api call. This
 * block is only intended to be called within the VehicleMonitoringDelivery Impl
 * block.
 */
impl VehicleActivity {
    /**
     * Creates a feed entity using this vehicle and gtfs data.
     */
    fn get_feed_entity(&self, gtfs: &Gtfs) -> Option<gtfs_rt::FeedEntity> {
        let trip = match gtfs.get_trip(
            &self
                .monitored_vehicle_journey
                .framed_vehicle_journey_ref
                .dated_vehicle_journey_ref,
        ) {
            Ok(x) => x,
            Err(_) => return None,
        };
        Some(FeedEntity {
            id: trip.id.clone(),
            is_deleted: Some(false),
            trip_update: None,
            vehicle: Some(self.get_vehicle_position(&trip)),
            alert: None,
            shape: None,
        })
    }

    /**
     * Creates a vehicle position (trip description, vehicle description, position,
     * time, etc.) using this vehicle and it's trip data.
     */
    fn get_vehicle_position(&self, trip: &Trip) -> gtfs_rt::VehiclePosition {
        VehiclePosition {
            trip: self.get_trip_descriptor(trip),
            vehicle: self.get_vehicle_descriptor(trip),
            position: self.get_position(),
            current_stop_sequence: None,
            stop_id: None,
            current_status: None,
            timestamp: Some(
                DateTime::parse_from_rfc3339(
                    &self.monitored_vehicle_journey.extensions.last_gps_fix,
                )
                .unwrap_or_default()
                .timestamp() as u64,
            ),
            congestion_level: None,
            occupancy_status: None,
            occupancy_percentage: None,
            multi_carriage_details: vec![self.get_carriage_details()],
        }
    }

    /**
     * Creates a trip descriptor (route number and id, direction, etc.) using this
     * vehicle and it's trip data.
     */
    fn get_trip_descriptor(&self, trip: &Trip) -> Option<gtfs_rt::TripDescriptor> {
        let direction_id = match trip.direction_id {
            Some(direction) => match direction {
                DirectionType::Outbound => 0,
                DirectionType::Inbound => 1,
            },
            None => return None,
        };
        Some(TripDescriptor {
            trip_id: Some(trip.id.clone()),
            route_id: Some(trip.route_id.clone()),
            direction_id: Some(direction_id),
            start_time: None,
            start_date: None,
            schedule_relationship: None,
        })
    }

    /**
     * Creates a vehicle descriptor (accessibility, vehicle id, etc.) using this
     * vehicle and it's trip data.
     */
    fn get_vehicle_descriptor(&self, trip: &Trip) -> Option<gtfs_rt::VehicleDescriptor> {
        let wheelchair_accessible = match trip.wheelchair_accessible {
            Availability::Available => Some(1),
            Availability::NotAvailable => Some(0),
            _ => None,
        };
        Some(VehicleDescriptor {
            id: Some(self.monitored_vehicle_journey.vehicle_ref.value.clone()),
            label: Some(self.monitored_vehicle_journey.vehicle_ref.value.clone()),
            license_plate: None,
            wheelchair_accessible: wheelchair_accessible,
        })
    }

    /**
     * Creates a position (latitude, longitude, bearing, speed, etc.) using this
     * vehicle.
     */
    fn get_position(&self) -> Option<gtfs_rt::Position> {
        Some(Position {
            latitude: self.monitored_vehicle_journey.vehicle_location.latitude,
            longitude: self.monitored_vehicle_journey.vehicle_location.longitude,
            bearing: Some(self.monitored_vehicle_journey.bearing),
            odometer: None,
            speed: None,
        })
    }

    /** Creates a carriage detail (id, label, etc.) using this vehicle. */
    fn get_carriage_details(&self) -> gtfs_rt::vehicle_position::CarriageDetails {
        CarriageDetails {
            id: Some(self.monitored_vehicle_journey.vehicle_ref.value.clone()),
            label: Some(self.monitored_vehicle_journey.vehicle_ref.value.clone()),
            occupancy_status: None,
            occupancy_percentage: None,
            carriage_sequence: Some(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_is_ok() {
        let locations = std::fs::read_to_string(
            "C:\\Users\\white\\dev\\catenary\\siri-gtfs_rt\\example-import-data\\locations.json",
        )
        .unwrap();
        let data = parse_data(locations);
        assert!(data.is_ok());
    }

    #[test]
    fn parse_data_gets_correct_numbers() {
        let locations = std::fs::read_to_string(
            "C:\\Users\\white\\dev\\catenary\\siri-gtfs_rt\\example-import-data\\locations.json",
        )
        .unwrap();
        let data = parse_data(locations).unwrap();
        let expected1: i32 = 830298;
        let expected2: f32 = 40.214482;
        let vehicle_0_origin_ref = data
            .clone()
            .vehicle_activity
            .unwrap()
            .get(0)
            .unwrap()
            .monitored_vehicle_journey
            .origin_ref
            .value
            .parse::<i32>()
            .unwrap();
        let vehicle_6_latitude = data
            .clone()
            .vehicle_activity
            .unwrap()
            .get(6)
            .unwrap()
            .monitored_vehicle_journey
            .vehicle_location
            .latitude;
        assert_eq!(vehicle_0_origin_ref, expected1);
        assert_eq!(vehicle_6_latitude, expected2)
    }

    #[tokio::test]
    async fn check_link_fetches() {
        let data = reqwest::get(String::from("http://api.rideuta.com/utartapi/VehicleMonitor/ByRoute?route=830x&onwardcalls=true&usertoken=") + API_KEY)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
        println!("{}", data);
        assert!(true);
    }
}
