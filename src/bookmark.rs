use iced::widget::image;
use iced::{clipboard, Command};

use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::bookmark_link::BookmarkLink;

use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;
use crate::movie_details::{Episode, TotalEpisode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub movie: TmdbMovie,

    pub current_episode: Episode,
    pub link: BookmarkLinkBox,
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
            link: BookmarkLinkBox::default(),
            show_details: false,
        }
    }
}
impl Bookmark {
    pub fn apply(&mut self, action: BookmarkMessage) -> Command<Message> {
        match action {
            BookmarkMessage::IncrE(details) => {
                if let Some(details) = details {
                    self.current_episode = details.next_episode(self.current_episode.clone());
                } else {
                    warn!("Can not incremend episode if movie details are not loaded");
                }
            }
            // BookmarkMessage::DecrE => self.current_episode = (self.current_episode - 1).max(1),
            BookmarkMessage::DecrE => {}
            BookmarkMessage::LinkInputChanged(new_input) => {
                if let BookmarkLinkBox::Input(s) = &mut self.link {
                    *s = new_input;
                }
            }
            BookmarkMessage::LinkInputSubmit => {
                if let BookmarkLinkBox::Input(s) = &mut self.link {
                    let link = BookmarkLink::new(s);
                    if let Ok(link) = link {
                        self.link = BookmarkLinkBox::Link(link);
                    } else {
                        error!("{:?} is not a valid link. Error {:?}", s, link)
                    }
                }
            }
            BookmarkMessage::LinkToClipboard(details) => {
                let BookmarkLinkBox::Link(link) = &self.link else {
                    return Command::none();
                };
                let url = match &self.current_episode {
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
                info!("copied {} to clipboard", &url);
                return Command::batch([
                    self.apply(BookmarkMessage::IncrE(details)),
                    clipboard::write::<Message>(url),
                ]);
            }
            BookmarkMessage::Remove => {}
            BookmarkMessage::ToggleDetails => {
                self.show_details = !self.show_details;
            }
            BookmarkMessage::RemoveLink => self.link = BookmarkLinkBox::Input(String::new()),
        }
        Command::none()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BookmarkLinkBox {
    Link(BookmarkLink),
    Input(String),
}
impl Default for BookmarkLinkBox {
    fn default() -> Self {
        BookmarkLinkBox::Input(String::new())
    }
}
