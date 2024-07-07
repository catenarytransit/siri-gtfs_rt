use gtfs_rt::*;
use serde::Deserialize;
use serde_json;

const API_KEY: &str = "UYEHABM01C9";

pub async fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}

#[derive(Deserialize, Clone)]
struct ServiceDelivery {
    #[serde(rename = "vehicleMonitoringDelivery")]
	vehicle_monitoring_delivery: VehicleMonitoringDelivery,
    #[serde(rename = "responseTimestamp")]
	response_timestamp: String,
}

#[derive(Deserialize, Clone)]
struct VehicleMonitoringDelivery {
    #[serde(rename = "vehicleActivity")]
	vehicle_activity: Option<Vec<VehicleActivity>>,
    #[serde(rename = "validUntil")]
	valid_until: String,
    #[serde(rename = "responseTimestamp")]
	response_timestamp: String,
}

#[derive(Deserialize, Clone)]
struct VehicleActivity {
    #[serde(rename = "monitoredVehicleJourney")]
	monitored_vehicle_journey: MonitoredVehicleJourney,
    #[serde(rename = "recordedAtTime")]
	recorded_at_time: String,
}

#[derive(Deserialize, Clone)]
struct MonitoredVehicleJourney {
	extensions: Extensions,
    #[serde(rename = "lineRef")]
	line_ref: LineRef,
    #[serde(rename = "framedVehicleJourneyRef")]
	framed_vehicle_journey_ref: FramedVehicleJourneyRef,
    #[serde(rename = "publishedLineName")]
	published_line_name: Vec<PublishedLineName>,
    #[serde(rename = "directionName")]
    direction_name: Vec<DirectionName>,
    #[serde(rename = "originRef")]
	origin_ref: OriginRef,
    #[serde(rename = "destinationRef")]
	destination_ref: DestinationRef,
    #[serde(rename = "destinationName")]
	destination_name: Vec<DestinationName>,
	monitored: bool,
    #[serde(rename = "vehicleLocation")]
	vehicle_location: VehicleLocation,
	bearing: f32,
    #[serde(rename = "progressStatus")]
	progress_status: Vec<ProgressStatus>,
    #[serde(rename = "courseOfJourneyRef")]
	course_of_journey_ref: CourseOfJourneyRef,
    #[serde(rename = "vehicleRef")]
	vehicle_ref: VehicleRef,
}

#[derive(Deserialize, Clone)]
struct Extensions {
    #[serde(rename = "lastGPSFix")]
	last_gps_fix: String,			//time
}

#[derive(Deserialize, Clone)]
struct LineRef {
	value: String,
}

#[derive(Deserialize, Clone)]
struct FramedVehicleJourneyRef {
    #[serde(rename = "dataFrameRef")]
	data_frame_ref: DataFrameRef,
    #[serde(rename = "datedVehicleJourneyRef")]
	dated_vehicle_journey_ref: i32,
}

#[derive(Deserialize, Clone)]
struct PublishedLineName {
	value: String,
}

#[derive(Deserialize, Clone)]
struct DirectionName {
	value: String,
}

#[derive(Deserialize, Clone)]
struct OriginRef {
	value: i32,				//origin station code
}

#[derive(Deserialize, Clone)]
struct DestinationRef {
	value: i32,				//destination station code
}

#[derive(Deserialize, Clone)]
struct DestinationName {
	value: String,				//destination station name
}

#[derive(Deserialize, Clone)]
struct VehicleLocation {
	latitude: f32,
	longitude: f32,
}

#[derive(Deserialize, Clone)]
struct ProgressStatus {
	value: String,
}

#[derive(Deserialize, Clone)]
struct CourseOfJourneyRef {
	value: i32,
}

#[derive(Deserialize, Clone)]
struct VehicleRef {
	value: u16,
}

#[derive(Deserialize, Clone)]
struct DataFrameRef {
    value: String,
}

// fn parse_data(data: String) -> Result<Siri, Box<dyn std::error::Error + Send + Sync>> {
//     // let data: Siri = from_str(&data)?;
//     // Ok(data)
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_is_ok() {
        // let data = String::from();
        // println!("{}", data);
        // let data = parse_data(data);
        // assert!(data.is_ok());
    }

    #[tokio::test]
    async fn check_link_fetches() {
        let data = reqwest::get("http://api.rideuta.com/utartapi/VehicleMonitor/ByRoute?route=830x&onwardcalls=true&usertoken=UYEHABM01C9")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
        println!("{}", data);
    }
}
