use gtfs_rt::*;
use serde::Deserialize;
use serde_json;

pub async fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}


#[derive(Deserialize, Clone)]
struct Siri {
    #[serde(rename = "ResponseTimestamp")]
	response_timestamp: String,
    #[serde(rename = "VehicleMonitoringDelivery")]
	vehicle_monitoring_delivery: VehicleMonitoringDelivery,
}

#[derive(Deserialize, Clone)]
struct VehicleMonitoringDelivery {
    #[serde(rename = "ResponseTimestamp")]
	response_timestamp: String,
    #[serde(rename = "ValidUntil")]
	valid_until: String,
    #[serde(rename = "VehicleActivity")]
	vehicle_activity: VehicleActivity,
}

#[derive(Deserialize, Clone)]
struct VehicleActivity {
    #[serde(rename = "RecordedAtTime")]
	recorded_at_time: String,
    #[serde(rename = "MonitoredVehicleJourney")]
	monitored_vehicle_journey: Vec<MonitoredVehicleJourney>,
}

#[derive(Deserialize, Clone)]
struct MonitoredVehicleJourney {
    #[serde(rename = "LineRef")]
	line_ref: String,
    #[serde(rename = "DirectionRef")]
	direction_ref: String,
    #[serde(rename = "FramedVehicleJourneyRef")]
	framed_vehicle_journey_ref: FramedVehicleJourneyRef,
    #[serde(rename = "PublishedLineName")]
	published_line_name: String,
    #[serde(rename = "OriginRef")]
	origin_ref: u32,
    #[serde(rename = "DestinationRef")]
	destination_ref: u32,
    #[serde(rename = "Monitored")]
	monitored: bool,
    #[serde(rename = "VehicleLocation")]
	vehicle_location: VehicleLocation,
    #[serde(rename = "ProgressRate")]
	progress_rate: u32,
    #[serde(rename = "CourseOfJourneyRef")]
	course_of_journey_ref: u32,
    #[serde(rename = "VehicleRef")]
	vehicle_ref: u32,
    #[serde(rename = "Extensions")]
	extensions: Extensions,
}

#[derive(Deserialize, Clone)]
struct FramedVehicleJourneyRef {
    #[serde(rename = "DataFrameRef")]
	data_frame_ref: String,
    #[serde(rename = "DatedVehicleJourneyRef")]
	dated_vehicle_journey_ref: u32,
}

#[derive(Deserialize, Clone)]
struct VehicleLocation {
    #[serde(rename = "Longitude")]
	longitude: f32,
    #[serde(rename = "Latitude")]
	latitude: f32,
}

#[derive(Deserialize, Clone)]
struct Extensions {
    #[serde(rename = "LastGPSFix")]
	last_gpsfix: String,
    #[serde(rename = "Scheduled")]
	scheduled: bool,
    #[serde(rename = "Bearing")]
	bearing: f32,
    #[serde(rename = "Speed")]
	speed: f32,
    #[serde(rename = "DestinationName")]
	destination_name: String,
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
