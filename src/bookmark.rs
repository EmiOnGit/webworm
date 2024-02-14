use iced::widget::image;
use iced::{clipboard, Command};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::bookmark_link::BookmarkLink;

use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    pub movie: TmdbMovie,
    pub current_episode: usize,
    pub current_season: usize,
    pub link: BookmarkLinkBox,
    pub finished: bool,
    #[serde(skip)]
    pub poster: Poster,
    #[serde(skip)]
    pub show_details: bool,
}
#[derive(Clone, Debug, Default)]
pub enum Poster {
    Image(image::Handle),
    #[default]
    None,
}
impl From<&TmdbMovie> for Bookmark {
    fn from(movie: &TmdbMovie) -> Self {
        Self {
            id: movie.id,
            movie: movie.clone(),
            current_episode: 1,
            current_season: 1,
            finished: false,
            link: BookmarkLinkBox::default(),
            poster: Poster::None,
            show_details: false,
        }
    }
}
impl Bookmark {
    pub fn apply(&mut self, action: BookmarkMessage) -> Command<Message> {
        match action {
            BookmarkMessage::IncrE(details) => {
                if let Some(details) = details {
                    self.current_episode =
                        (self.current_episode + 1).min(details.last_published().episode_number)
                } else {
                    self.current_episode += 1
                }
            }
            BookmarkMessage::IncrS => self.current_season += 1,
            BookmarkMessage::DecrE => self.current_episode = (self.current_episode - 1).max(1),
            BookmarkMessage::DecrS => self.current_season = (self.current_season - 1).max(1),
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
                        warn!("{:?} is not a valid link. Error {:?}", s, link)
                    }
                }
            }
            BookmarkMessage::LinkToClipboard(details) => {
                let BookmarkLinkBox::Link(link) = &self.link else {
                    return Command::none();
                };
                let url = link.url(self.current_episode, self.current_season);
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
