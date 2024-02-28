use serde::{Deserialize, Serialize};

use crate::id::MovieId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TmdbMovie {
    pub id: MovieId,
    pub genre_ids: Vec<usize>,
    pub overview: String,
    vote_average: f32,
    pub original_name: String,
    pub name: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
}

impl TmdbMovie {
    pub fn rating(&self) -> u8 {
        (self.vote_average * 10.) as u8
    }
    pub fn matches_filter(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
            || self.original_name.to_lowercase().contains(filter)
    }
}
