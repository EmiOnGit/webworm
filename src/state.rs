use iced::Command;
use tracing::debug;

use crate::filter::Filter;
use crate::id::{EpisodeId, MovieId};
use crate::link::Link;
use crate::message::{Message, ShiftPressed};
use crate::save::SavedState;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::movie::TmdbMovie;
use crate::movie_details::{EpisodeDetails, MovieDetails};

use crate::bookmark::{Bookmark, Poster};
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum InputKind {
    SearchField,
    EpisodeInput,
    SeasonInput,
    LinkInput,
}
impl InputKind {
    pub fn index(&self) -> usize {
        match self {
            InputKind::SearchField => 0,
            InputKind::EpisodeInput => 1,
            InputKind::SeasonInput => 2,
            InputKind::LinkInput => 3,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct InputCaches([String; 4]);

impl IndexMut<InputKind> for InputCaches {
    fn index_mut(&mut self, index: InputKind) -> &mut Self::Output {
        &mut self.0[index.index()]
    }
}

impl Index<InputKind> for InputCaches {
    type Output = String;

    fn index(&self, index: InputKind) -> &Self::Output {
        &self.0[index.index()]
    }
}
#[derive(Debug, Default)]
pub struct GuiState {
    pub input_caches: InputCaches,
    pub filter: Filter,
    pub dirty: bool,
    pub saving: bool,
    pub shift_pressed: ShiftPressed,
}
#[derive(Debug, Default)]
pub struct State {
    pub gui: GuiState,
    pub movies: Vec<TmdbMovie>,
    pub movie_details: HashMap<MovieId, MovieDetails>,
    pub movie_posters: HashMap<MovieId, Poster>,
    pub episode_details: HashMap<EpisodeId, EpisodeDetails>,
    pub links: HashMap<MovieId, Link>,
    pub bookmarks: Vec<Bookmark>,
}
impl State {
    pub fn save(&mut self, saved: bool) -> Command<Message> {
        if !saved {
            self.gui.dirty = true;
        }

        if self.gui.dirty && !self.gui.saving {
            self.gui.dirty = false;
            self.gui.saving = true;
            debug!("saving state");
            Command::perform(
                SavedState {
                    bookmarks: self.bookmarks.clone(),
                    links: self.links.clone(),
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
    pub(crate) fn set_detail_input_caches(&mut self, movie_id: MovieId) {
        let Some(bookmark) = self.get_bookmark(movie_id) else {
            return;
        };
        let episode = bookmark.current_episode.clone();
        let link = self
            .links
            .get(&movie_id)
            .map(|link| link.string_link.clone())
            .unwrap_or_default();
        self.gui.input_caches[InputKind::EpisodeInput] = episode.episode().to_string();
        self.gui.input_caches[InputKind::SeasonInput] = episode.season().to_string();
        self.gui.input_caches[InputKind::LinkInput] = link;
    }
}
