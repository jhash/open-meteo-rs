#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use open_meteo_rs::Client;
use open_meteo_rs::jiff::Timestamp;
use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub const FORECAST_PATH: &str = "/v1/forecast";
pub const ARCHIVE_PATH: &str = "/v1/archive";
pub const AIR_QUALITY_PATH: &str = "/v1/air-quality";
pub const SEARCH_PATH: &str = "/v1/search";

pub const DST_FORECAST: &str = include_str!("../fixtures/forecast_auckland_dst.json");
pub const MINUTELY_FORECAST: &str = include_str!("../fixtures/forecast_minutely_15.json");
pub const MODELS_FORECAST: &str = include_str!("../fixtures/forecast_models.json");
pub const EXTRA_FORECAST: &str = include_str!("../fixtures/forecast_extra.json");
pub const ARCHIVE: &str = include_str!("../fixtures/archive_berlin.json");
pub const AIR_QUALITY: &str = include_str!("../fixtures/air_quality_berlin.json");
pub const SEARCH: &str = include_str!("../fixtures/search_auckland.json");
pub const EMPTY_SEARCH: &str = include_str!("../fixtures/search_empty.json");
pub const SPARSE_SEARCH: &str = include_str!("../fixtures/search_sparse.json");
pub const REFUSED: &str = include_str!("../fixtures/refused.json");

pub type Hit = HashMap<String, String>;

#[derive(Debug, Clone)]
struct Canned {
    status: u16,
    body: &'static str,
}

#[derive(Debug, Default)]
struct Recorded {
    replies: HashMap<String, Canned>,
    hits: Vec<(String, Hit)>,
}

type Shared = Arc<Mutex<Recorded>>;

pub struct Upstream {
    url: String,
    recorded: Shared,
    task: JoinHandle<()>,
}

impl Upstream {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("the stub binds a loopback port");
        let address = listener.local_addr().expect("the stub has an address");
        let recorded = Shared::default();
        let serving = recorded.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(answer(stream, serving.clone()));
            }
        });
        Self {
            url: format!("http://{address}"),
            recorded,
            task,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn client(&self) -> Client {
        self.builder().build().expect("the stub client builds")
    }

    pub fn builder(&self) -> open_meteo_rs::ClientBuilder {
        Client::builder()
            .forecast_base(self.url())
            .archive_base(self.url())
            .air_quality_base(self.url())
            .geocoding_base(self.url())
    }

    pub fn reply(&self, path: &str, status: u16, body: &'static str) -> &Self {
        self.lock()
            .replies
            .insert(path.to_owned(), Canned { status, body });
        self
    }

    pub fn hits(&self, path: &str) -> Vec<Hit> {
        self.lock()
            .hits
            .iter()
            .filter(|(hit_path, _)| hit_path == path)
            .map(|(_, query)| query.clone())
            .collect()
    }

    pub fn only_hit(&self, path: &str) -> Hit {
        let mut hits = self.hits(path);
        assert_eq!(hits.len(), 1, "one request to {path}");
        hits.remove(0)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Recorded> {
        self.recorded.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Upstream {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn answer(mut stream: TcpStream, recorded: Shared) {
    let mut head = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !head.windows(4).any(|window| window == b"\r\n\r\n") {
        match stream.read(&mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(read) => head.extend_from_slice(&buffer[..read]),
        }
    }
    let head = String::from_utf8_lossy(&head);
    let target = head.split_whitespace().nth(1).unwrap_or("/");
    let url = Url::parse(&format!("http://stub{target}")).expect("the request target parses");
    let query: Hit = url.query_pairs().into_owned().collect();
    let canned = {
        let mut recorded = recorded.lock().unwrap_or_else(PoisonError::into_inner);
        recorded.hits.push((url.path().to_owned(), query));
        recorded.replies.get(url.path()).cloned()
    }
    .unwrap_or(Canned {
        status: 404,
        body: "{\"reason\":\"no canned reply\",\"error\":true}",
    });
    let response = format!(
        "HTTP/1.1 {} Stub\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        canned.status,
        canned.body.len(),
        canned.body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

pub fn ms(timestamp: Timestamp) -> i64 {
    timestamp.as_millisecond()
}

pub fn listed(hit: &Hit, name: &str) -> Vec<String> {
    hit.get(name)
        .map(|value| value.split(',').map(str::to_owned).collect())
        .unwrap_or_default()
}
