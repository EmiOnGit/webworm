use iced::alignment::{Horizontal, Vertical};
use iced::theme::{self};
use iced::widget::{button, column, image, row, text, text_input};
use iced::{clipboard, Command, Length};
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::bookmark_link::BookmarkLink;

use crate::gui::{FONT_SIZE, FONT_SIZE_HEADER, INPUT_LINK_ID};
use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;
use crate::movie_details::MovieDetails;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    movie: TmdbMovie,
    current_episode: usize,
    current_season: usize,
    link: BookmarkLinkBox,
    pub finished: bool,
    #[serde(skip)]
    pub poster: Poster,
    #[serde(skip)]
    show_details: bool,
    pub poster_path: String,
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
            poster_path: movie.poster_path.clone(),
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
    pub fn view(&self, _i: usize, details: Option<&MovieDetails>) -> Element<BookmarkMessage> {
        let body = row![
            self.picture_view(Length::FillPortion(3)),
            column![
                row![column![
                    text(self.movie.name.as_str())
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::FillPortion(1))
                        .size(FONT_SIZE_HEADER)
                        .style(theme::Text::Default),
                    row![
                        if let Some(details) = &details {
                            let left =
                                details.last_published().episode_number as i32 - self.current_episode as i32;
                            if left < 0 {
                                text(format!("Something weird happened. You are {} episodes ahead of the release", left.abs()))
                            } else if left != 0 {
                                text(format!(
                                    "LEFT: {}",
                                    details.last_published().episode_number - self.current_episode
                                ))
                            } else {
                                text("No episodes left to watch")
                            }
                            .width(Length::FillPortion(1))
                        } else {
                            text("details not loaded").width(Length::FillPortion(1))
                        },
                        text(if self.current_season > 1 {
                            format!(
                                "PROGRESS: {ep}E · {s}S",
                                ep = self.current_episode,
                                s = self.current_season
                            )
                        } else {
                            format!("PROGRESS: {ep}E", ep = self.current_episode,)
                        })
                        .width(Length::FillPortion(1))
                        .horizontal_alignment(Horizontal::Right)
                    ]
                    .align_items(Alignment::Center)
                ],],
                self.link_view(details)
            ]
            .width(Length::FillPortion(5)),
            button(if self.show_details { "↓" } else { "↑" })
                .padding(30.)
                .style(theme::Button::Secondary)
                .on_press(BookmarkMessage::ToggleDetails),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        if self.show_details {
            column![
                body,
                row![
                    column![
                        text(format!("VOTE: {:.1}/10", self.movie.vote_average)),
                        text(format!("POPULARITY: {:.0}", self.movie.popularity))
                    ]
                    .width(Length::FillPortion(1)),
                    column![
                        row![
                            iced::widget::container(row![
                                button("↑")
                                    .style(theme::Button::Secondary)
                                    .on_press(BookmarkMessage::IncrE(details.cloned()))
                                    .padding(10),
                                text(format!(
                                    "E {} · S {}",
                                    self.current_episode, self.current_season
                                ))
                                .vertical_alignment(Vertical::Bottom),
                                button("↓")
                                    .style(theme::Button::Secondary)
                                    .padding(10)
                                    .on_press(BookmarkMessage::DecrE)
                            ])
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                            iced::widget::container(
                                button("X")
                                    .on_press(BookmarkMessage::Remove)
                                    .padding(30)
                                    .style(theme::Button::Secondary)
                            )
                            .align_x(Horizontal::Right),
                        ]
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                        text(&self.movie.overview)
                    ]
                    .width(Length::FillPortion(5))
                ],
            ]
            .into()
        } else {
            body.into()
        }
    }
    fn picture_view(&self, width: Length) -> Element<BookmarkMessage> {
        if let Poster::Image(img) = &self.poster {
            image::viewer(img.clone())
                .width(Length::Fixed(500.))
                .height(Length::Fixed(200.))
                .into()
        } else {
            iced::widget::text("IMG").width(width).into()
        }
    }
    fn link_view(&self, details: Option<&MovieDetails>) -> Element<BookmarkMessage> {
        match &self.link {
            BookmarkLinkBox::Link(l) => iced::widget::button(l.string_link.as_str())
                .on_press(BookmarkMessage::LinkToClipboard(details.cloned()))
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
