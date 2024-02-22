use crate::movie::TmdbMovie;
use anyhow::Result;
use core::fmt;
use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::warn;
#[derive(Debug, Clone)]
pub enum RequestType {
    TvSearch { query: String },
    TvDetails { id: usize },
    Poster { id: usize, path: String },
}
impl RequestType {
    pub fn url(&self) -> String {
        let base_url = "https://api.themoviedb.org/3/";
        let rest = "language=en-US&page=1";
        let body = match self {
            RequestType::TvSearch { query } => {
                let query_cleaned = query.replace(' ', "%20");
                format!("search/tv?&query={query_cleaned}&")
            }
            RequestType::TvDetails { id } => format!("tv/{id}?"),
            RequestType::Poster { id: _, path } => {
                return format!("https://image.tmdb.org/t/p/w500/{path}")
            }
        };
        format!("{base_url}{body}{rest}")
    }
}
pub async fn send_request(config: TmdbConfig, request: RequestType) -> Result<String> {
    let url = request.url();
    warn!("send request with {url}");
    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(config.token.clone())
        .build()
        .unwrap();
    let response = Client::new().execute(request)?;
    let data: String = response.text().unwrap();
    Ok(data)
}
pub async fn send_byte_request(config: TmdbConfig, request: RequestType) -> Result<Vec<u8>> {
    let url = request.url();
    warn!("send request with {url}");
    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(config.token.clone())
        .build()
        .unwrap();
    let response = Client::new().execute(request)?;
    let data: Vec<u8> = response.bytes().unwrap().into_iter().collect();
    Ok(data)
}
#[derive(Clone)]
pub struct TmdbConfig {
    token: String,
}
impl fmt::Debug for TmdbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TmdbConfig [Confidential]")
    }
}
impl TmdbConfig {
    pub fn new(content: &str) -> Option<TmdbConfig> {
        // let content = async_std::fs::read_to_string("cred.md").await?;
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect();
        Some(TmdbConfig {
            token: lines[0].to_owned(),
        })
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct TmdbResponse {
    pub results: Vec<TmdbMovie>,
}
