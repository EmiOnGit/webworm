use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{bookmark::Bookmark, movie::TmdbMovie, movie_details::Episode};
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct EpisodeId(pub MovieId, pub Episode);

#[derive(Clone, Hash, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MovieId(usize);
impl MovieId {
    pub fn id(&self) -> usize {
        self.0
    }
}
impl Display for MovieId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id {}", self.0)
    }
}
pub trait MovieIndex<T> {
    fn with_id(&self, id: MovieId) -> Option<&T>;
    fn with_id_mut(&mut self, id: MovieId) -> Option<&mut T>;
}
impl MovieIndex<TmdbMovie> for Vec<TmdbMovie> {
    fn with_id(&self, id: MovieId) -> Option<&TmdbMovie> {
        self.iter().find(|movie| movie.id == id)
    }

    fn with_id_mut(&mut self, id: MovieId) -> Option<&mut TmdbMovie> {
        self.iter_mut().find(|movie| movie.id == id)
    }
}
impl MovieIndex<Bookmark> for Vec<Bookmark> {
    fn with_id(&self, id: MovieId) -> Option<&Bookmark> {
        self.iter().find(|bookmark| bookmark.movie.id == id)
    }

    fn with_id_mut(&mut self, id: MovieId) -> Option<&mut Bookmark> {
        self.iter_mut().find(|bookmark| bookmark.movie.id == id)
    }
}
impl<T> MovieIndex<T> for HashMap<MovieId, T> {
    fn with_id(&self, id: MovieId) -> Option<&T> {
        self.get(&id)
    }

    fn with_id_mut(&mut self, id: MovieId) -> Option<&mut T> {
        self.get_mut(&id)
    }
}
