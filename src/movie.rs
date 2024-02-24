use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TmdbMovie {
    pub id: MovieId,
    pub genre_ids: Vec<usize>,
    pub overview: String,
    pub vote_average: f32,
    original_name: String,
    pub name: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MovieMessage {
    ToggleBookmark,
}

impl TmdbMovie {
    pub fn rating(&self) -> u8 {
        (self.vote_average * 10.) as u8
    }
}

#[derive(Clone, Hash, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MovieId(usize);
impl Display for MovieId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id {}", self.0)
    }
}
impl MovieId {
    pub fn id(&self) -> usize {
        self.0
    }
}
