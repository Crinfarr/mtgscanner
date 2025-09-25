mod card;
use std::{collections::VecDeque, error::Error, io::Write, path::Path, time::Instant};
use tracing::{Instrument, instrument, span};

use reqwest::{
    Client, StatusCode,
    header::{ACCEPT, CONTENT_LENGTH},
};
use serde::Deserialize;
use serde_with::chrono::{self, DateTime};
use tracing::{Level, event, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::card::{Card, CardImageStatus};

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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), RuntimeError> {
    tracing_subscriber::FmtSubscriber::new()
        .with(LevelFilter::from_level(Level::DEBUG))
        .init();
    let client = Client::builder()
        .user_agent("io.crinfarr.scanner-bot")
        .build()?;
    if Path::new("./all_cards.json").exists() {
        event!(Level::INFO, "Using local card data");
    } else {
        // event!(Level::INFO, "Fetching latest revision of scryfall data");
        event!(Level::INFO, "Downloading bulk data index");
        let response = client
            .get("https://api.scryfall.com/bulk-data")
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .json::<BulkResponse>()
            .await?;
        event!(Level::INFO, "Downloading bulk data");
        let st = Instant::now();
        let mut download_stream = client
            .get(
                response
                    .data
                    .iter()
                    .find(|obj| obj.bulk_type == "all_cards")
                    .unwrap()
                    .download_uri
                    .clone(),
            )
            .send()
            .await?;
        let mut f_handle = std::fs::File::create("./all_cards.json").unwrap();
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
        let elapsed = Instant::now().duration_since(st);
        event!(
            Level::INFO,
            "Downloaded {} bytes in {}.{} seconds",
            c_length,
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
    }
    let st = Instant::now();
    event!(Level::INFO, "Loading file...");
    let content = String::from_utf8(std::fs::read("./all_cards.json")?)?;
    event!(Level::INFO, "Parsing...");
    let mut cards = Box::pin(serde_json::from_str::<VecDeque<card::Card>>(&content)?);
    let elapsed = Instant::now().duration_since(st);
    event!(
        Level::INFO,
        "Parsed {} cards in {}.{}s",
        content.lines().count() - 2,
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    // let shared_client = reqwest::Client::default();
    while let Some(card) = cards.pop_front() {
        tokio::task::spawn(async |card:Card| -> () {
            if card.image_status != CardImageStatus::HighresScan{
                return;
            } else {
                if let Some(imgs) = card.image_uris {
                    if let Some(fullsize_img) = imgs.large {
                        event!(Level::DEBUG, "Downloading image for {}/{}", card.set, card.collector_number);
                        let res = reqwest::get(fullsize_img).await;
                        if let Ok(_response) = res {
                            event!(Level::INFO, "Downloaded {}/{}", card.set, card.collector_number);
                            return;
                        }
                        if let Err(e) = res {
                            event!(Level::ERROR, "Error when downloading {}/{}: [{:?}]: {}", card.set, card.collector_number,e.source().unwrap(), e);
                            return;
                        }
                    } else {
                        event!(Level::WARN, "Card {}/{} claims highres-scan, but fullsize image is not available", card.set, card.collector_number)
                    }
                } else {
                    event!(Level::WARN, "Card {}/{} claims highres-scan, but no images are available", card.set, card.collector_number);
                    return;
                }
            }
        }(card).instrument(span!(Level::INFO, "CardDownloader")));
    }
    Ok(())
}
