use iced::alignment::Horizontal;
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
use crate::movie_details::MovieDetails;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    name: String,
    current_episode: usize,
    current_season: usize,
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
                self.current_episode += 1;
                info!("copied {} to clipboard", &url);
                return clipboard::write::<Message>(url);
            }
            BookmarkMessage::Remove => {}
        }
        Command::none()
    }
    pub fn view(&self, _i: usize, details: Option<&MovieDetails>) -> Element<BookmarkMessage> {
        iced::widget::container(
            row![
                text(self.name.as_str())
                    .width(Length::FillPortion(1))
                    .style(theme::Text::Default),
                column![
                    row![
                        column![
                            button("+1")
                                .style(theme::Button::Secondary)
                                .on_press(BookmarkMessage::IncrE)
                                .width(Length::Fixed(30.)),
                            button("-1")
                                .style(theme::Button::Secondary)
                                .on_press(BookmarkMessage::DecrE)
                                .width(Length::Fixed(30.)),
                        ],
                        column![text(format!(
                            "E {}, S {}",
                            self.current_episode, self.current_season
                        )),],
                        column![
                            button("+1")
                                .style(theme::Button::Secondary)
                                .on_press(BookmarkMessage::IncrS)
                                .width(Length::Fixed(30.)),
                            button("-1")
                                .style(theme::Button::Secondary)
                                .on_press(BookmarkMessage::DecrS)
                                .width(Length::Fixed(30.)),
                        ],
                        self.details_latest(details),
                        iced::widget::container(
                            button("X")
                                .on_press(BookmarkMessage::Remove)
                                .padding(10)
                                .style(theme::Button::Secondary)
                        )
                        .width(Length::Fill)
                        .align_x(Horizontal::Right),
                    ]
                    .spacing(20)
                    .align_items(Alignment::Center),
                    row![self.link_view()]
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                ]
                .width(Length::FillPortion(4))
                .spacing(10),
            ]
            .align_items(Alignment::Center),
        )
        .style(theme::Container::Box)
        .width(Length::Fill)
        .padding(10)
        .into()
    }
    fn details_latest(&self, details: Option<&MovieDetails>) -> Element<BookmarkMessage> {
        if let Some(details) = details {
            let latest = details.last_published();
            text(format!(
                "Latest: E {}, S {}",
                latest.episode_number, latest.season_number
            ))
            .into()
        } else {
            iced::widget::Space::with_width(Length::Fixed(10.)).into()
        }
    }
    fn link_view(&self) -> Element<BookmarkMessage> {
        match &self.link {
            BookmarkLinkBox::Link(l) => iced::widget::button(l.string_link.as_str())
                .on_press(BookmarkMessage::LinkToClipboard)
                .style(theme::Button::Secondary)
                .width(Length::Fill)
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
