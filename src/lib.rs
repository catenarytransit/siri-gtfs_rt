use gtfs_rt::*;
use serde::Deserialize;
use serde_xml_rs::from_str;

pub fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}

#[derive(Deserialize)]
struct SiriMessage {
    #[serde(rename = "ResponseTimestamp")]
    response_timestamp: String,
    #[serde(rename = "VehicleMonitoringDelivery")]
    vehicle_monitoring_delivery: VehicleMonitoringDelivery,
}

#[derive(Deserialize)]
struct VehicleMonitoringDelivery {
    #[serde(rename = "ResponseTimestamp")]
    response_timestamp: String,
    #[serde(rename = "ValidUntil")]
    valid_until: String,
    #[serde(rename = "VehicleActivity")]
    vehicle_activity: VehicleActivity,
}

#[derive(Deserialize)]
struct VehicleActivity {
    #[serde(rename = "RecordedAtTime")]
    time_recorded: String,
    #[serde(rename = "MonitoredVehicleJourney")]
    monitored_vehicles: Vec<MonitoredVehicleJourney>,
}

#[derive(Deserialize)]
struct MonitoredVehicleJourney {
    #[serde(rename = "LineRef")]
    route_number: String,
    #[serde(rename = "DirectionRef")]
    direction_name: String,
    #[serde(rename = "FramedVehicleJourneyRef")]
    framed_vehicle_journey_refs: FramedVehicleJourneyRef,
    #[serde(rename = "PublishedLineName")]
    route_name: String,
    #[serde(rename = "OriginRef")]
    origin_ref: u32,
    #[serde(rename = "DestinationRef")]
    destination_ref: u32,
    #[serde(rename = "Monitored")]
    monitored: bool,
    #[serde(rename = "VehicleLocation")]
    vehicle_location: VehicleLocation,
    #[serde(rename = "ProgressRate")]
    progress_rate: u8,
    #[serde(rename = "CourseOfJourneyRef")]
    course_of_journey_ref: u32,
    #[serde(rename = "VehicleRef")]
    vehicle_number: u32,
    #[serde(rename = "Extensions")]
    extensions: Extensions,
}

#[derive(Deserialize)]
struct FramedVehicleJourneyRef {
    #[serde(rename = "DateFrameRef")]
    data_frame_ref: String,
    #[serde(rename = "DatedVehicleJourneyRef")]
    dated_vehicle_journey_ref: u32,
}

#[derive(Deserialize)]
struct VehicleLocation {
    #[serde(rename = "Longitude")]
    longitude: f32,
    #[serde(rename = "Latitude")]
    latitude: f32,
}

#[derive(Deserialize)]
struct Extensions {
    #[serde(rename = "LastGPSFix")]
    last_gps_fix: String,
    #[serde(rename = "Scheduled")]
    scheduled: bool,
    #[serde(rename = "Bearing")]
    bearing: f32,
    #[serde(rename = "Speed")]
    speed: f32,
    #[serde(rename = "DestinationName")]
    destination_name: String,
}

fn parse_data(data: String) -> Result<SiriMessage, Box<dyn std::error::Error + Send + Sync>> {
    let data: SiriMessage = from_str(&data)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_is_ok() {
        let data = String::from(
            r#"
            <Siri version="1.3">
            <ResponseTimestamp>2024-04-29T21:40:18.063479-06:00</ResponseTimestamp>
            <VehicleMonitoringDelivery version="1.3">
            <ResponseTimestamp>2024-04-29T21:40:18.063479-06:00</ResponseTimestamp>
            <ValidUntil>2024-04-29T21:40:28.063479-06:00</ValidUntil>
            <VehicleActivity>
            <RecordedAtTime>2024-04-29T21:40:18.063479-06:00</RecordedAtTime>
            <MonitoredVehicleJourney>
            <LineRef>2</LineRef>
            <DirectionRef>TO U HOSPITAL</DirectionRef>
            <FramedVehicleJourneyRef>
            <DataFrameRef>2024-04-29T00:00:00-06:00</DataFrameRef>
            <DatedVehicleJourneyRef>5308041</DatedVehicleJourneyRef>
            </FramedVehicleJourneyRef>
            <PublishedLineName>200 SOUTH</PublishedLineName>
            <OriginRef>125332</OriginRef>
            <DestinationRef>118161</DestinationRef>
            <Monitored>True</Monitored>
            <VehicleLocation>
            <Longitude>-111.90993133333333</Longitude>
            <Latitude>40.764484833333334</Latitude>
            </VehicleLocation>
            <ProgressRate>1</ProgressRate>
            <CourseOfJourneyRef>511113</CourseOfJourneyRef>
            <VehicleRef>23109</VehicleRef>
            <Extensions>
            <LastGPSFix>2024-04-29T21:40:09.763</LastGPSFix>
            <Scheduled>False</Scheduled>
            <Bearing>163.7</Bearing>
            <Speed>0</Speed>
            <DestinationName>University Hospital</DestinationName>
            </Extensions>
            </MonitoredVehicleJourney>
            </VehicleActivity>
            </VehicleMonitoringDelivery>
            </Siri>"#,
        );
        println!("{}", data);
    }
}
