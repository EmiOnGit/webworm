use iced::Command;
use tracing::debug;

use crate::filter::Filter;
use crate::id::{EpisodeId, MovieId};
use crate::link::BookmarkLinkBox;
use crate::message::{Message, ShiftPressed};
use crate::save::SavedState;
use std::collections::HashMap;

use crate::movie::TmdbMovie;
use crate::movie_details::{EpisodeDetails, MovieDetails};
use crate::tmdb::TmdbConfig;

use crate::bookmark::{Bookmark, Poster};

#[derive(Debug, Default)]
pub struct State {
    pub input_value: String,
    pub filter: Filter,
    pub movies: Vec<TmdbMovie>,
    pub movie_details: HashMap<MovieId, MovieDetails>,
    pub movie_posters: HashMap<MovieId, Poster>,
    pub episode_details: HashMap<EpisodeId, EpisodeDetails>,
    pub links: HashMap<MovieId, BookmarkLinkBox>,
    pub bookmarks: Vec<Bookmark>,
    pub dirty: bool,
    pub saving: bool,
    pub tmdb_config: Option<TmdbConfig>,
    pub shift_pressed: ShiftPressed,
}
impl State {
    pub fn save(&mut self, saved: bool) -> Command<Message> {
        if !saved {
            self.dirty = true;
        }

        if self.dirty && !self.saving {
            self.dirty = false;
            self.saving = true;
            debug!("saving state");
            Command::perform(
                SavedState {
                    bookmarks: self.bookmarks.clone(),
                    links: self.links.clone(),
                    // We ignore it anyway since we save it in a text file
                    tmdb_config: None,
                }
                .save(),
                Message::Saved,
            )
        } else {
            Command::none()
        }
    }
    pub fn get_bookmark(&self, movie_id: MovieId) -> Option<&Bookmark> {
        self.bookmarks
            .iter()
            .find(|bookmark| bookmark.movie.id == movie_id)
    }
}
