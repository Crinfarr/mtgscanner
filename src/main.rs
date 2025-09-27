mod card;
use image::DynamicImage;
use image_hasher::ImageHash;
use std::{
    collections::BTreeSet, error::Error, fs::File, io::{BufRead, BufReader, Write}, path::Path, sync::Arc, time::Instant
};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::bytes::Bytes;
use tracing::{Instrument, span};

use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_LENGTH},
};
use serde::Deserialize;
use serde_with::chrono::{self, DateTime};
use tracing::{Level, event};

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

const TARGET_BULK_TYPE: &'static str = "all_cards";
const LOG_LEVEL: Level = Level::DEBUG;
const MAX_DOWNLOAD_THREADS: u16 = 200;
const MAX_HASH_THREADS: u16 = 1000;

struct ImgRecord {
    ihash:ImageHash,
    cardrep:Card
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), RuntimeError> {
    tracing_subscriber::fmt().with_max_level(LOG_LEVEL).init();
    let client = Arc::from(
        Client::builder()
            .user_agent("io.crinfarr.scanner-bot")
            .build()?,
    );

    if Path::new(&format!("./{TARGET_BULK_TYPE}.json")).exists() {
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
                    .find(|obj| obj.bulk_type == TARGET_BULK_TYPE)
                    .unwrap()
                    .download_uri
                    .clone(),
            )
            .send()
            .await?;
        let mut f_handle = File::create(format!("./{TARGET_BULK_TYPE}.json")).unwrap();
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

    let mut join_set: JoinSet<()> = JoinSet::default();

    let f_reader = std::fs::File::open(format!("./{TARGET_BULK_TYPE}.json"))?;
    event!(Level::DEBUG, "Trying to lock card file");
    f_reader.lock()?;
    event!(Level::DEBUG, "File locked");
    let r_buf = BufReader::new(f_reader);

    let sem_parallel_dls = Arc::new(Semaphore::new(MAX_DOWNLOAD_THREADS as usize));
    let sem_parallel_hashes = Arc::new(Semaphore::new(MAX_HASH_THREADS as usize));
    let mut threadnum = 0;
    for maybe_line in r_buf.lines() {
        threadnum += 1;
        let line = maybe_line?;
        if line == "[" || line == "]" {
            continue;
        }
        let sem_load_ref = sem_parallel_dls.clone();
        let sem_hash_ref = sem_parallel_hashes.clone();
        let client_ref = client.clone();
        let card = serde_json::from_str::<Card>(&line[0..line.len() - 1])
            .or_else(|_| serde_json::from_str::<Card>(&line))?;
        event!(Level::DEBUG, "Starting thread for {}", card.name);
        join_set.spawn(
            async move {
                let hasher = image_hasher::HasherConfig::new()
                    .hash_alg(image_hasher::HashAlg::DoubleGradient)
                    .hash_size(16, 16)
                    .to_hasher();
                event!(Level::TRACE, "Waiting on permit");
                let permit = sem_load_ref.acquire().await.unwrap();
                event!(Level::TRACE, "Permit captured");
                let mut imgs:BTreeSet<ImgRecord> = BTreeSet::new();
                match card.image_status {
                    CardImageStatus::HighresScan => {
                        event!(Level::TRACE, "Highres scan available");
                        match card.image_uris {
                            Some(uris) => {
                                if let Some(uri) = uris.large {
                                    event!(Level::DEBUG, "Fetching image");
                                    match client_ref.get(uri).send().await {
                                        Ok(res) => {
                                            event!(Level::INFO, "Download complete");
                                            event!(Level::DEBUG, "Loading image");
                                            let img_bytes = res.bytes().await.unwrap();
                                            drop(permit);
                                            match image::load_from_memory(&img_bytes) {
                                                Ok(img) => {
                                                    let _permit = sem_hash_ref.acquire().await.unwrap();
                                                    event!(Level::DEBUG, "Loaded image");
                                                    let hash = hasher.hash_image(&img);
                                                    imgs.insert(ImgRecord {cardrep: card, ihash: hash});

                                                },
                                                Err(e) => {
                                                    event!(Level::WARN, "Failed to load image, err {e}");
                                                    return;
                                                }
                                            }
                                        }
                                        Err(e) => event!(
                                            Level::WARN,
                                            "Error downloading: {} ({:?})",
                                            e,
                                            e.source()
                                        ),
                                    };
                                } else {
                                    event!(
                                        Level::WARN,
                                        "Card claimed highres but no large image available"
                                    );
                                }
                            }
                            None => match card.card_faces {
                                Some(faces) => {
                                    let mut face_num = 0;
                                    for face in faces {
                                        face_num += 1;
                                        if let Some(uris) = face.image_uris {
                                            if let Some(uri) = uris.large {
                                                event!(Level::DEBUG, "Fetching image");
                                                match client_ref.get(uri).send().await {
                                                    Ok(_) => {
                                                        event!(Level::INFO, "Download complete")
                                                    }
                                                    Err(e) => event!(
                                                        Level::WARN,
                                                        "Error downloading: {} ({:?})",
                                                        e,
                                                        e.source()
                                                    ),
                                                };
                                            }
                                        } else {
                                            event!(
                                                Level::WARN,
                                                "Claimed highres but face {} had no images",
                                                face_num
                                            );
                                        }
                                    }
                                }
                                None => {
                                    event!(
                                        Level::WARN,
                                        "Card claimed highres but no faces were available"
                                    );
                                }
                            },
                        }
                    }
                    quality => {
                        event!(
                            Level::DEBUG,
                            "{}/{}: No highres scan available, max quality {:?}",
                            card.set,
                            card.collector_number,
                            quality
                        );
                        return;
                    }
                }
                drop(permit);
                let _permit = sem_hash_ref.acquire().await.unwrap();
            }
            .instrument(span!(
                Level::INFO,
                "downloader thread",
                cardname = card.name,
                thread_id = threadnum
            )),
        );
    }
    join_set.join_all().await;
    let elapsed = Instant::now().duration_since(st);
    event!(
        Level::INFO,
        "Loaded all cards in {}.{} seconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    Ok(())
}
