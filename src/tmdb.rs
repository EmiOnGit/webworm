use crate::{
    id::{EpisodeId, MovieId},
    movie::TmdbMovie,
    movie_details::SeasonEpisode,
};
use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::info;

const BEAR: &str = "eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiJjNjk0MWZiYjQxYmM4ZjEyYjNjZmFmNzU5YTg1ZmM2NiIsInN1YiI6IjY1YjY0OTQ2NjBjNTFkMDE4NGQyNDhlNiIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.sFwFG4LtWHtO5rbRMEwCNS7thN4n-NrDThzAxRo5rHQ";

#[derive(Debug, Clone)]
pub enum RequestType {
    TvSearch { query: String },
    TvDetails { id: MovieId },
    Poster { id: MovieId, path: String },
    EpisodeDetails { id: EpisodeId },
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
            RequestType::TvDetails { id } => format!("tv/{}?", id.id()),
            RequestType::Poster { id: _, path } => {
                return format!("https://image.tmdb.org/t/p/w500/{path}")
            }
            RequestType::EpisodeDetails { id } => {
                let seasonal = match &id.1 {
                    crate::movie_details::Episode::Seasonal(e) => e.clone(),
                    crate::movie_details::Episode::Total(e) => SeasonEpisode {
                        episode_number: e.episode,
                        season_number: 1,
                    },
                };
                format!(
                    "tv/{movie_id}/season/{s}/episode/{ep}?",
                    movie_id = id.0.id(),
                    s = seasonal.season_number,
                    ep = seasonal.episode_number,
                )
            }
        };
        format!("{base_url}{body}{rest}")
    }
}
pub async fn send_request(request: RequestType) -> Result<String> {
    let url = request.url();
    info!("send request with {}", &url[8..]);
    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(BEAR)
        .build()
        .unwrap();
    let response = Client::new().execute(request)?;
    let data: String = response.text().unwrap();
    Ok(data)
}
pub async fn send_byte_request(request: RequestType) -> Result<Vec<u8>> {
    let url = request.url();
    info!("send request with {}", &url[8..]);
    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(BEAR)
        .build()
        .unwrap();
    let response = Client::new().execute(request)?;
    let data: Vec<u8> = response.bytes().unwrap().into_iter().collect();
    Ok(data)
}
#[derive(Debug, Clone, Deserialize)]
pub struct TmdbResponse {
    pub results: Vec<TmdbMovie>,
}
