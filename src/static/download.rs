use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;

async fn makereqandsave(url: &str, filename: &str, client: &ReqwestClient) {
    let mut req = client.get(url);
    req = req.header("X-RapidAPI-Host", "transloc-api-1-2.p.rapidapi.com");
    req = req.header(
        "X-RapidAPI-Key",
        "X7rzqy7Zx8mshBtXeYQjrv0aLyrYp1HBttujsnJ6BgNQxIMetU",
    );

    let response = req.send().await;

    match response {
        Ok(response) => {
            // println!("response: {:?}", response);

            let string = &response.text().await.unwrap();

            let mut file = File::create(filename).unwrap();
            file.write_all(string.as_bytes()).unwrap();

            println!("Saved");
        }
        Err(e) => {
            println!("error: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    //--header 'X-RapidAPI-Host: transloc-api-1-2.p.rapidapi.com' \
    //	--header 'X-RapidAPI-Key: X7rzqy7Zx8mshBtXeYQjrv0aLyrYp1HBttujsnJ6BgNQxIMetU'

    let client = ReqwestClient::new();

    // get stops
    //https://transloc-api-1-2.p.rapidapi.com/stops.json?agencies=1039&callback=call

    makereqandsave(
        "https://transloc-api-1-2.p.rapidapi.com/stops.json?agencies=1039",
        "staticfiles/stops.json",
        &client,
    )
    .await;

    //get segments
    //https://transloc-api-1-2.p.rapidapi.com/segments.json?agencies=1039&callback=call

    makereqandsave(
        "https://transloc-api-1-2.p.rapidapi.com/segments.json?agencies=1039",
        "staticfiles/segments.json",
        &client,
    )
    .await;

    //get routes
    //https://transloc-api-1-2.p.rapidapi.com/routes.json?agencies=1039

    makereqandsave(
        "https://transloc-api-1-2.p.rapidapi.com/routes.json?agencies=1039",
        "staticfiles/routes.json",
        &client,
    )
    .await;

    //get agency info
    //https://transloc-api-1-2.p.rapidapi.com/agencies.json?agencies=1039

    makereqandsave(
        "https://transloc-api-1-2.p.rapidapi.com/agencies.json?agencies=1039",
        "staticfiles/agencies.json",
        &client,
    )
    .await;
}
