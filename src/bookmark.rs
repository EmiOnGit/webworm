use iced::theme::{self};
use iced::widget::{button, column, row, text, text_input};
use iced::{clipboard, Command, Length};
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::bookmark_link::BookmarkLink;

use crate::gui::{FONT_SIZE, INPUT_LINK_ID};
use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    name: String,
    current_episode: usize,
    current_season: usize,
    latest_season: usize,
    latest_episode: usize,
    pub finished: bool,
    link: BookmarkLinkBox,
}
impl From<&TmdbMovie> for Bookmark {
    fn from(movie: &TmdbMovie) -> Self {
        Self {
            id: movie.id,
            name: movie.name.clone(),
            current_episode: 1,
            current_season: 1,
            latest_season: 1,
            latest_episode: 1,
            finished: false,
            link: BookmarkLinkBox::default(),
        }
    }
}
impl Bookmark {
    pub fn apply(&mut self, action: BookmarkMessage) -> Command<Message> {
        match action {
            BookmarkMessage::IncrE => self.current_episode += 1,
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
            BookmarkMessage::LinkToClipboard => {
                let BookmarkLinkBox::Link(link) = &self.link else {
                    return Command::none();
                };
                let url = link.url(self.current_episode, self.current_season);
                info!("copied {} to clipboard", &url);
                return clipboard::write::<Message>(url);
            }
        }
        Command::none()
    }
    pub fn view(&self, _i: usize) -> Element<BookmarkMessage> {
        let info_column = column![text(self.name.as_str()).style(theme::Text::Default),];
        iced::widget::container(column![
            row![
                info_column,
                column![
                    text(format!(
                        "E {}, S {}",
                        self.current_episode, self.current_season
                    )),
                    text(format!(
                        "E {}, S {}",
                        self.latest_episode, self.latest_season
                    )),
                ],
                column![
                    button("E + 1").on_press(BookmarkMessage::IncrE),
                    button("S + 1").on_press(BookmarkMessage::IncrS),
                ],
                column![
                    button("E - 1").on_press(BookmarkMessage::DecrE),
                    button("S - 1").on_press(BookmarkMessage::DecrS),
                ],
            ]
            .spacing(20)
            .align_items(Alignment::Center),
            self.link_view()
        ])
        .style(theme::Container::Box)
        .width(Length::Fill)
        .padding(20.)
        .into()
    }
    fn link_view(&self) -> Element<BookmarkMessage> {
        match &self.link {
            BookmarkLinkBox::Link(l) => iced::widget::button(l.string_link.as_str())
                .on_press(BookmarkMessage::LinkToClipboard)
                .into(),
            BookmarkLinkBox::Input(s) => text_input("Link:", s)
                .id(INPUT_LINK_ID.clone())
                .on_input(BookmarkMessage::LinkInputChanged)
                .on_submit(BookmarkMessage::LinkInputSubmit)
                .padding(15)
                .size(FONT_SIZE)
                .into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
enum BookmarkLinkBox {
    Link(BookmarkLink),
    Input(String),
}
impl Default for BookmarkLinkBox {
    fn default() -> Self {
        BookmarkLinkBox::Input(String::new())
    }
}
