use gtfs_rt::*;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use chrono::{DateTime, FixedOffset, ParseResult};

const API_KEY: &str = "UYEHABM01C9";

pub async fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>>
{
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
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

struct Vehicle {
	last_GPS_update: DateTime<FixedOffset>,
	line_number: String,
	date: DateTime<FixedOffset>,
	vehicle_journey_ref: i32,
	line_name: Vec<String>,
	direction_name: Vec<String>,
	origin_code: i32,
	destination_code: i32,
	destination_name: Vec<String>,
	is_monitored: bool,
	latitude: f32,
	longitude: f32,
	bearing: f32,
	progress_status: Vec<String>,
	course_of_journey_ref: i32,
	vehicle_number: u16,
    time_recorded: DateTime<FixedOffset>,
}

impl VehicleMonitoringDelivery {
    fn get_response_timestamp(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.response_timestamp)
    }

    fn get_valid_until(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.valid_until)
    }

    fn get_vehicles(&self) -> Option<Vec<Vehicle>> {
        let vehicle_activities = &self.vehicle_activity.clone()?;
        let mut vehicles: Vec<Vehicle> = vec![];
        for vehicle_activity in vehicle_activities {
            let mut line_names: Vec<String> = vec![];
            for line_name in vehicle_activity.monitored_vehicle_journey.published_line_name.clone() {
                line_names.append(&mut vec![line_name.value]);
            }
            let mut direction_names: Vec<String> = vec![];
            for direction_name in vehicle_activity.monitored_vehicle_journey.direction_name.clone() {
                direction_names.append(&mut vec![direction_name.value]);
            }
            let mut destination_names: Vec<String> = vec![];
            for destination_name in vehicle_activity.monitored_vehicle_journey.destination_name.clone() {
                destination_names.append(&mut vec![destination_name.value]);
            }
            let mut progress_statuses: Vec<String> = vec![];
            for progress_status in vehicle_activity.monitored_vehicle_journey.progress_status.clone() {
                progress_statuses.append(&mut vec![progress_status.value]);
            }
            let vehicle = Vehicle {
                last_GPS_update: DateTime::parse_from_rfc3339(&vehicle_activity.monitored_vehicle_journey.extensions.last_gps_fix).unwrap_or_default(),
                line_number: vehicle_activity.monitored_vehicle_journey.line_ref.value.clone(),
                date: DateTime::parse_from_rfc3339(&vehicle_activity.monitored_vehicle_journey.framed_vehicle_journey_ref.data_frame_ref.value).unwrap_or_default(),
                vehicle_journey_ref: vehicle_activity.monitored_vehicle_journey.framed_vehicle_journey_ref.dated_vehicle_journey_ref.parse::<i32>().unwrap_or_default(),
                line_name: line_names,
                direction_name: direction_names,
                origin_code: vehicle_activity.monitored_vehicle_journey.origin_ref.value.parse::<i32>().unwrap_or_default(),
                destination_code: vehicle_activity.monitored_vehicle_journey.destination_ref.value.parse::<i32>().unwrap_or_default(),
                destination_name: destination_names,
                is_monitored: vehicle_activity.monitored_vehicle_journey.monitored,
                latitude: vehicle_activity.monitored_vehicle_journey.vehicle_location.latitude,
                longitude: vehicle_activity.monitored_vehicle_journey.vehicle_location.longitude,
                bearing: vehicle_activity.monitored_vehicle_journey.bearing,
                progress_status: progress_statuses,
                course_of_journey_ref: vehicle_activity.monitored_vehicle_journey.course_of_journey_ref.value.parse::<i32>().unwrap_or_default(),
                vehicle_number: vehicle_activity.monitored_vehicle_journey.vehicle_ref.value.parse::<u16>().unwrap_or_default(),
                time_recorded: DateTime::parse_from_rfc3339(&vehicle_activity.recorded_at_time).unwrap_or_default(),
            };
            vehicles.append(&mut vec![vehicle]);
        }
        Some(vehicles)
    }
}

fn parse_data(data: String) -> Result<Siri, Box<dyn std::error::Error + Send + Sync>> {
    let data: Siri = from_str(&data)?;
    Ok(data)
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
            .service_delivery
            .vehicle_monitoring_delivery
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
            .service_delivery
            .vehicle_monitoring_delivery
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
