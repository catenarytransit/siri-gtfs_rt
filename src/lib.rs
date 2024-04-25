use gtfs_rt::*;
use serde::Deserialize;

pub fn get_gtfs_rt() -> Result<gtfs_rt::FeedMessage, Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}
#[derive(Deserialize)]
struct SiriMessage {

}

fn parse_data(data: String)  -> Result<SiriMessage, Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid String",
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
}

