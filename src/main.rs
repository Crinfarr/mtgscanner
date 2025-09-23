mod card;
use std::{any::Any, fs::File, io::Write, path::Path, time::Instant};

use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_LENGTH},
};
use serde::Deserialize;
use serde_with::chrono::{self, DateTime};
use tokio::runtime::Runtime;
use tracing::{Level, event, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt};

#[derive(Deserialize)]
#[allow(unused)]
struct BulkEntry {
    object: String,
    id: String,
    #[serde(rename = "type")]
    bulk_type: String,
    updated_at: DateTime<chrono::FixedOffset>,
    uri: String,
    name: String,
    description: String,
    size: u32,
    download_uri: String,
    content_type: String,
    content_encoding: String,
}
#[derive(Deserialize)]
#[allow(unused)]
struct BulkResponse {
    object: String,
    has_more: bool,
    data: Vec<BulkEntry>,
}

#[derive(Debug)]
#[allow(unused)]
enum RuntimeError {
    ReqwestError(reqwest::Error),
    IOError(std::io::Error),
    TextDecodeError(std::string::FromUtf8Error),
    JSONDecodeError(serde_json::Error),
}
impl From<reqwest::Error> for RuntimeError {
    fn from(err: reqwest::Error) -> RuntimeError {
        RuntimeError::ReqwestError(err)
    }
}
impl From<std::io::Error> for RuntimeError {
    fn from(err: std::io::Error) -> RuntimeError {
        RuntimeError::IOError(err)
    }
}
impl From<std::string::FromUtf8Error> for RuntimeError {
    fn from(err: std::string::FromUtf8Error) -> RuntimeError {
        RuntimeError::TextDecodeError(err)
    }
}
impl From<serde_json::Error> for RuntimeError {
    fn from(err: serde_json::Error) -> RuntimeError {
        RuntimeError::JSONDecodeError(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    tracing_subscriber::FmtSubscriber::new()
        .with(LevelFilter::from_level(Level::DEBUG))
        .init();
    event!(Level::INFO, "Fetching latest revision of scryfall data");
    let client = Client::builder()
        .user_agent("io.crinfarr.scanner-bot")
        .build()?;
    if Path::new("./unique_artwork.json").exists() {
        event!(Level::INFO, "Using local card data");
    } else {
        event!(Level::INFO, "Downloading bulk data index");
        let response = client
            .get("https://api.scryfall.com/bulk-data")
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .json::<BulkResponse>()
            .await?;
        event!(Level::INFO, "Downloading bulk data");
        let mut st = Instant::now();
        let mut download_stream = client
            .get(
                response
                    .data
                    .iter()
                    .find(|obj| obj.bulk_type == "unique_artwork")
                    .unwrap()
                    .download_uri
                    .clone(),
            )
            .send()
            .await?;
        let mut f_handle = std::fs::File::create("./unique_artwork.json").unwrap();
        let c_length = str::parse::<u32>(
            download_stream
                .headers()
                .get(CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .expect("No content_length header"),
        )
        .expect("Failed to parse content_length");
        let mut acc: u32 = 0;
        while let Some(chunk) = download_stream.chunk().await? {
            acc += chunk.len() as u32;
            print!(
                "Downloading: {}%   \r",
                ((f64::from(acc) / f64::from(c_length as u32)) * 100f64).floor()
            );
            f_handle.write(&chunk).expect("Failed to write file");
        }
        let mut elapsed = Instant::now().duration_since(st);
        event!(
            Level::INFO,
            "Downloaded {} bytes in {}.{} seconds",
            c_length,
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
    }
    let mut st = Instant::now();
    let content = String::from_utf8(std::fs::read("./unique_artwork.json")?)?;
    let cards = serde_json::from_str::<Vec<card::Card>>(&content)?;
    let mut elapsed = Instant::now().duration_since(st);
    event!(
        Level::INFO,
        "Parsed {} cards in {}.{}s",
        content.lines().count() - 2,
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    st = Instant::now();
    for card in cards {
        print!("{}               \r", card.collector_number)
    }
    elapsed = Instant::now().duration_since(st);
    event!(Level::INFO, "Took {}.{} secs to loop through", elapsed.as_secs(), elapsed.subsec_millis());
    Ok(())
}
