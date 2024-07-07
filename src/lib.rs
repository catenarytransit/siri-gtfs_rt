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
struct ServiceDelivery {
	VehicleMonitoringDelivery: VehicleMonitoringDelivery,
	ResponseTimestamp: String,		//time
}

#[derive(Deserialize, Clone)]
struct VehicleMonitoringDelivery {
	VehicleActivity: Option<Vec<VehicleActivity>>,
	ValidUntil: String, 			//time
	ResponseTimestamp: String, 		//time
}

#[derive(Deserialize, Clone)]
struct VehicleActivity {
	MonitoredVehicleJourney: MonitoredVehicleJourney,
	RecordedAtTime: String,			//time
}

#[derive(Deserialize, Clone)]
struct MonitoredVehicleJourney {
	Extensions: Extensions,
	LineRef: LineRef,
	FramedVehicleJourneyRef: FramedVehicleJourneyRef,
	PublishedLineName: Vec<PublishedLineName>,
	DirectionName: Vec<DirectionName>,
	OriginRef: OriginRef,
	DestinationRef: DestinationRef,
	DestinationName: Vec<DestinationName>,
	Monitored: bool,
	VehicleLocation: VehicleLocation,
	Bearing: f32,
	ProgressStatus: Vec<ProgressStatus>,
	CourseOfJourneyRef: CourseOfJourneyRef,
	VehicleRef: VehicleRef,
}

#[derive(Deserialize, Clone)]
struct Extensions {
	LastGPSFix: String,			//time
}

#[derive(Deserialize, Clone)]
struct LineRef {
	value: String,
}

#[derive(Deserialize, Clone)]
struct FramedVehicleJourneyRef {
	DataFrameRef: DataFrameRef,
	DatedVehicleJourneyRef: i32,
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
