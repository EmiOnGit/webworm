use iced::widget::image;
use iced::{clipboard, Command};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::link::Link;

use crate::message::{BookmarkMessage, LinkMessage, Message, ShiftPressed};
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
            BookmarkMessage::Remove => {}
            BookmarkMessage::ToggleDetails => {
                self.show_details = !self.show_details;
            }
        }
        Command::none()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BookmarkLinkBox {
    Link(Link),
    Input(String),
}
impl BookmarkLinkBox {
    pub fn apply(&mut self, bookmark: &mut Bookmark, message: LinkMessage) -> Command<Message> {
        match message {
            LinkMessage::LinkInputSubmit => {
                if let BookmarkLinkBox::Input(s) = self {
                    let link = Link::new(s);
                    debug!("Link was submitted and parsed to {:?}", link);
                    if let Ok(link) = link {
                        *self = BookmarkLinkBox::Link(link);
                    } else {
                        error!("{:?} is not a valid link. Error {:?}", s, link)
                    }
                } else {
                    warn!(
                        "received a LinkInputSubmit message the bookmark has no LinkInput {:?}",
                        self
                    );
                }
            }
            LinkMessage::LinkToClipboard(details, shift) => {
                let BookmarkLinkBox::Link(link) = &self else {
                    return Command::none();
                };
                let url = match &bookmark.current_episode {
                    Episode::Seasonal(e) => {
                        if link.has_season() {
                            link.url(e.episode_number, e.season_number)
                        } else {
                            let Some(details) = &details else {
                                error!("load details before copying to clipboard");
                                return Command::none();
                            };
                            link.url(details.as_total_episodes(&e).episode, 1)
                        }
                    }
                    Episode::Total(e) => {
                        if link.has_season() {
                            let Some(details) = &details else {
                                error!("load details before copying to clipboard");
                                return Command::none();
                            };
                            let e = details.as_seasonal_episode(&e);
                            link.url(e.episode_number, e.season_number)
                        } else {
                            link.url(e.episode, 1)
                        }
                    }
                };
                debug!("copied {} to clipboard", &url);
                return if shift == ShiftPressed::True {
                    debug!("Since shift was pressed the bookmark is not increased");
                    Command::batch([clipboard::write::<Message>(url)])
                } else {
                    Command::batch([
                        bookmark.apply(BookmarkMessage::IncrE(details)),
                        clipboard::write::<Message>(url),
                    ])
                };
            }
            LinkMessage::LinkInputChanged(new_input) => {
                if let BookmarkLinkBox::Input(s) = self {
                    *s = new_input;
                }
            }
            LinkMessage::RemoveLink => {
                *self = BookmarkLinkBox::Input(String::new());
            }
        }
        Command::none()
    }
}
impl Default for BookmarkLinkBox {
    fn default() -> Self {
        BookmarkLinkBox::Input(String::new())
    }
}
