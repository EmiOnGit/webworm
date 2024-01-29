use crate::{bookmark::Bookmark, movie::TmdbMovie};
use anyhow::Result;
use core::fmt;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub async fn queue_tv_series(config: TmdbConfig, query: String) -> Result<String> {
    let request = request(&config, &query);

    let response = Client::new().execute(request)?;

    let data: String = response.text().unwrap();
    Ok(data)
}
fn request(config: &TmdbConfig, query: &str) -> reqwest::blocking::Request {
    let query_clean = query.replace(' ', "%20");
    let base_url = "https://api.themoviedb.org/3/search/tv?";
    let rest = "language=en-US&page=1";
    let url = format!("{base_url}&query={query_clean}&{rest}");

    Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(config.token.clone())
        .build()
        .unwrap()
}
#[derive(Clone)]
pub struct TmdbConfig {
    token: String,
    _key: String,
}
impl fmt::Debug for TmdbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TmdbConfig [Confidential]")
    }
}
impl TmdbConfig {
    pub async fn new() -> Result<TmdbConfig> {
        let content = async_std::fs::read_to_string("cred.md").await?;
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect();
        Ok(TmdbConfig {
            token: lines[0].to_owned(),
            _key: lines[1].to_owned(),
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbResponse {
    page: usize,
    results: Vec<TmdbMovie>,
    total_pages: usize,
    total_results: usize,
}
impl TmdbResponse {
    /// Returns the movies from the `TmdbRequest`.
    /// The bookmarks are used to set the `is_bookmark` flag of the corresponding movie.
    pub fn movies(&mut self, bookmarks: &[Bookmark]) -> &Vec<TmdbMovie> {
        for movie in &mut self.results {
            movie.set_bookmark(bookmarks);
        }
        &self.results
    }
}
