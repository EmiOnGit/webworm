use iced::Command;

use crate::filter::Filter;
use crate::message::Message;
use crate::save::SavedState;
use std::collections::HashMap;

use crate::movie::TmdbMovie;
use crate::movie_details::MovieDetails;
use crate::tmdb::TmdbConfig;

use crate::bookmark::Bookmark;

#[derive(Debug, Default)]
pub struct State {
    pub input_value: String,
    pub filter: Filter,
    pub movies: Vec<TmdbMovie>,
    pub movie_details: HashMap<usize, MovieDetails>,
    pub bookmarks: Vec<Bookmark>,
    pub dirty: bool,
    pub saving: bool,
    pub tmdb_config: Option<TmdbConfig>,
    pub debug: DebugState,
}
#[derive(Default, Debug, PartialEq)]
pub enum DebugState {
    Debug,
    #[default]
    Release,
}
impl State {
    pub fn save(&mut self, saved: bool) -> Command<Message> {
        if !saved {
            self.dirty = true;
        }

        if self.dirty && !self.saving {
            self.dirty = false;
            self.saving = true;
            Command::perform(
                SavedState {
                    bookmarks: self.bookmarks.clone(),
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
