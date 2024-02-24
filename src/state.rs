use iced::Command;
use tracing::debug;

use crate::filter::Filter;
use crate::message::{Message, ShiftPressed};
use crate::save::SavedState;
use std::collections::HashMap;

use crate::movie::{MovieId, TmdbMovie};
use crate::movie_details::MovieDetails;
use crate::tmdb::TmdbConfig;

use crate::bookmark::{Bookmark, BookmarkLinkBox, Poster};

#[derive(Debug, Default)]
pub struct State {
    pub input_value: String,
    pub filter: Filter,
    pub movies: Vec<TmdbMovie>,
    pub movie_details: HashMap<MovieId, MovieDetails>,
    pub movie_posters: HashMap<MovieId, Poster>,
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
}
