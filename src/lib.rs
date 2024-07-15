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

impl VehicleMonitoringDelivery {
    fn get_response_timestamp(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.response_timestamp)
    }

    fn get_valid_until(&self) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.valid_until)
    }
}

impl VehicleActivity {
    fn get_feed_entity(&self) -> gtfs_rt::FeedEntity {
        FeedEntity {
            id: todo!(),
            is_deleted: Some(false),
            trip_update: None,
            vehicle: Some(self.get_vehicle_position()),
            alert: todo!(),
            shape: todo!(),
        }
    }

    fn get_vehicle_position(&self) -> gtfs_rt::VehiclePosition {
        VehiclePosition {
            trip: todo!(),
            vehicle: todo!(),
            position: todo!(),
            current_stop_sequence: todo!(),
            stop_id: todo!(),
            current_status: todo!(),
            timestamp: todo!(),
            congestion_level: todo!(),
            occupancy_status: todo!(),
            occupancy_percentage: todo!(),
            multi_carriage_details: todo!(),
        }
    }

    fn get_trip_descriptor(&self) -> gtfs_rt::TripDescriptor {
        TripDescriptor {
            trip_id: Some(String::from("")),
            route_id: Some(self.monitored_vehicle_journey.line_ref.value.clone()),
            direction_id: Some(0),
            start_time: todo!(),
            start_date: todo!(),
            schedule_relationship: todo!(),
        }
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
