use iced::widget::image;
use iced::Command;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;
use crate::movie_details::{Episode, TotalEpisode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub movie: TmdbMovie,
    pub current_episode: Episode,
    /// True if the current episode is already watched
    pub finished: bool,
    #[serde(skip)]
    pub show_details: bool,
}
#[derive(Clone, Debug)]
pub enum Poster {
    Image(image::Handle),
}
impl From<&TmdbMovie> for Bookmark {
    fn from(movie: &TmdbMovie) -> Self {
        Self {
            movie: movie.clone(),
            current_episode: Episode::Total(TotalEpisode { episode: 1 }),
            finished: false,
            show_details: false,
        }
    }
}
impl Bookmark {
    pub fn apply(&mut self, action: BookmarkMessage) -> Command<Message> {
        match action {
            BookmarkMessage::IncrE(details) => {
                if let Some(details) = details {
                    debug!("increment bookmark episode {:?}", self);
                    let next_episode = details.next_episode(self.current_episode.clone());
                    self.finished = next_episode == self.current_episode;
                    self.current_episode = next_episode;
                } else {
                    warn!(
                        "Can not increment episode as the movie details are not loaded id: {}",
                        self.movie.id
                    );
                }
            }
            BookmarkMessage::DecrE(details) => {
                if let Some(details) = details {
                    debug!("decrement bookmark episode {:?}", self);
                    if self.finished {
                        debug!("Since the bookmark was on finished state, finish flag was removed");
                        self.finished = false;
                    } else {
                        let previous_episode =
                            details.previous_episode(self.current_episode.clone());
                        self.current_episode = previous_episode;
                        self.finished = false;
                    }
                } else {
                    warn!(
                        "Can not decrement episode as the movie details are not loaded id: {}",
                        self.movie.id
                    );
                }
            }
            BookmarkMessage::ToggleDetails => {
                self.show_details = !self.show_details;
            }
        }
        Command::none()
    }
}
