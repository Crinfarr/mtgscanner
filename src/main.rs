mod card;
use std::{fs::File, io::Write, path::Path, time::Instant};

use reqwest::{
    Client, Error,
    header::{ACCEPT, CONTENT_LENGTH},
};
use serde::Deserialize;
use serde_with::chrono::{self, DateTime};
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

#[tokio::main]
async fn main() -> Result<(), Error> {
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
        let st = Instant::now();
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
        event!(
            Level::INFO,
            "Downloaded {} bytes in {}.{} seconds",
            c_length,
            Instant::now().duration_since(st).as_secs(),
            Instant::now().duration_since(st).subsec_millis()
        );
    }
    let cards = serde_json::from_str::<Vec<card::Card>>(
        str::from_utf8(&std::fs::read("./unique_artworks.json").unwrap_or(vec![b'[', b']']))
            .unwrap_or("[]"),
    ).unwrap_or(vec![]);
    for card in cards {
    }
    Ok(())
}
