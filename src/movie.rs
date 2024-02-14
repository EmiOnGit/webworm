use crate::bookmark::Bookmark;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TmdbMovie {
    pub id: usize,
    pub genre_ids: Vec<usize>,
    pub overview: String,
    pub vote_average: f32,
    original_name: String,
    pub name: String,
    pub popularity: f32,
    pub poster_path: String,

    #[serde(skip)]
    pub is_bookmark: bool,
}

#[derive(Debug, Clone)]
pub enum MovieMessage {
    ToggleBookmark,
}

impl TmdbMovie {
    pub fn rating(&self) -> u8 {
        (self.vote_average * 10.) as u8
    }
    pub fn set_bookmark(&mut self, bookmarks: &[Bookmark]) {
        self.is_bookmark = bookmarks.iter().any(|bookmark| bookmark.id == self.id);
    }

    pub fn update(&mut self, message: MovieMessage) {
        match message {
            MovieMessage::ToggleBookmark => {
                self.is_bookmark = !self.is_bookmark;
            }
        }
    }
}
