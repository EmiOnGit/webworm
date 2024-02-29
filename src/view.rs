use iced::alignment::{Horizontal, Vertical};
use iced::theme::{self};
use iced::widget::{button, column, image, row, text, text_input, Image, Row, Space};
use iced::Length;
use iced::{Alignment, Element};

use crate::bookmark::{Bookmark, Poster};
use crate::filter::Filter;
use crate::gui::{icon, FONT_SIZE, FONT_SIZE_HEADER, INPUT_LINK_ID};
use crate::id::MovieId;
use crate::link::BookmarkLinkBox;
use crate::message::{BookmarkMessage, LinkMessage, Message, ShiftPressed};
use crate::movie::TmdbMovie;
use crate::movie_details::{Episode, EpisodeDetails, MovieDetails};

impl Bookmark {
    pub fn card_view<'a>(
        &'a self,
        details: Option<&MovieDetails>,
        link: &'a BookmarkLinkBox,
        poster: Option<&'a Poster>,
    ) -> Element<Message> {
        let picture_row = row![
            Space::with_width(Length::Fill),
            picture_view(self.movie.id, poster, Length::FillPortion(3)),
            button("PLAY")
                .on_press(Message::LinkMessage(
                    self.movie.id,
                    LinkMessage::LinkToClipboard(details.cloned(), ShiftPressed::Unknown)
                ))
                .width(Length::Fill)
        ];
        let progress = text(format!("PROGRESS: {}", self.current_episode.as_info_str()))
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let latest = if let Some(details) = &details {
            if let Some(last) = details.last_published() {
                text(format!(
                    "LATEST: {}",
                    Into::<Episode>::into(last.episode).as_info_str()
                ))
            } else {
                text("no latest episode")
            }
        } else {
            text("details not loaded")
        }
        .width(Length::Fill)
        .horizontal_alignment(Horizontal::Center);
        column![
            picture_row,
            text(self.movie.name.as_str())
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Fill)
                .size(FONT_SIZE_HEADER),
            progress,
            latest
        ]
        .into()
    }
    pub fn view<'a>(
        &'a self,
        details: Option<&MovieDetails>,
        link: &'a BookmarkLinkBox,
        poster: Option<&'a Poster>,
    ) -> Element<Message> {
        let body = row![
            picture_view(self.movie.id, poster, Length::FillPortion(3)),
            column![
                row![column![
                    text(self.movie.name.as_str())
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::FillPortion(1))
                        .size(FONT_SIZE_HEADER)
                        .style(theme::Text::Default),
                    row![
                        if let Some(details) = &details {
                            if let Some(latest) = details.last_published() {
                                text(format!(
                                    "LATEST: {}",
                                    Into::<Episode>::into(latest.episode).as_info_str()
                                ))
                                .width(Length::FillPortion(1))
                            } else {
                                text("No latest episode")
                            }
                        } else {
                            text("details not loaded").width(Length::FillPortion(1))
                        },
                        text(format!("PROGRESS: {}", self.current_episode.as_info_str()))
                            .width(Length::FillPortion(1))
                            .horizontal_alignment(Horizontal::Right)
                    ]
                    .align_items(Alignment::Center)
                ],],
                link.link_view(self.movie.id, details)
            ]
            .width(Length::FillPortion(5)),
            button(if self.show_details { "↑" } else { "↓" })
                .padding(30.)
                .style(theme::Button::Secondary)
                .on_press(Message::BookmarkMessage(
                    self.movie.id,
                    BookmarkMessage::ToggleDetails
                )),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        if self.show_details {
            column![
                body,
                row![
                    column![text(format!("POPULARITY: {:.0}", self.movie.popularity))]
                        .width(Length::FillPortion(1)),
                    column![
                        row![
                            iced::widget::container(row![
                                button("↑")
                                    .style(theme::Button::Secondary)
                                    .on_press(Message::BookmarkMessage(
                                        self.movie.id,
                                        BookmarkMessage::IncrE(details.cloned())
                                    ))
                                    .padding(10),
                                text(self.current_episode.as_info_str().to_string())
                                    .vertical_alignment(Vertical::Bottom),
                                button("↓")
                                    .style(theme::Button::Secondary)
                                    .padding(10)
                                    .on_press(Message::BookmarkMessage(
                                        self.movie.id,
                                        BookmarkMessage::DecrE(details.cloned())
                                    ))
                            ])
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                            iced::widget::container(
                                button("---")
                                    .on_press(Message::LinkMessage(
                                        self.movie.id,
                                        LinkMessage::RemoveLink
                                    ))
                                    .padding(30)
                                    .style(theme::Button::Secondary)
                            )
                            .align_x(Horizontal::Right),
                            iced::widget::container(
                                button("X")
                                    .on_press(Message::RemoveBookmark(self.movie.id))
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
}
impl BookmarkLinkBox {
    fn link_view(&self, id: MovieId, details: Option<&MovieDetails>) -> Element<Message> {
        match &self {
            BookmarkLinkBox::Link(l) => iced::widget::button(l.string_link.as_str())
                .on_press(Message::LinkMessage(
                    id,
                    LinkMessage::LinkToClipboard(details.cloned(), ShiftPressed::Unknown),
                ))
                .style(theme::Button::Secondary)
                .width(Length::Fill)
                .into(),
            BookmarkLinkBox::Input(s) => text_input("Link:", s)
                .id(INPUT_LINK_ID.clone())
                .on_input(move |s| Message::LinkMessage(id, LinkMessage::LinkInputChanged(s)))
                .on_submit(Message::LinkMessage(id, LinkMessage::LinkInputSubmit))
                .width(Length::Fill)
                .padding(15)
                .size(FONT_SIZE)
                .into(),
        }
    }
}
impl TmdbMovie {
    pub fn view<'a>(&'a self, poster: Option<&'a Poster>) -> Element<Message> {
        let info_column = column![
            text(self.name.as_str()).style(theme::Text::Default),
            text(format!("Rating: {0}%", self.rating())),
        ];
        let description: String = self
            .overview
            .split_whitespace()
            .take(30)
            .collect::<Vec<&str>>()
            .join(" ");
        iced::widget::container(
            row![
                picture_view(self.id, poster, Length::FillPortion(3)),
                info_column.width(Length::FillPortion(2)),
                text(description + "...")
                    .width(Length::FillPortion(6))
                    .style(theme::Text::Default),
                button(icon('✍'))
                    .on_press(Message::AddBookmark(self.id))
                    .padding(10)
                    .width(Length::FillPortion(1))
                    .style(theme::Button::Text),
            ]
            .spacing(20)
            .align_items(Alignment::Start),
        )
        .style(theme::Container::Transparent)
        .width(Length::Fill)
        .padding(20.)
        .into()
    }
}
fn picture_view(id: MovieId, poster: Option<&Poster>, width: Length) -> Element<Message> {
    if let Some(Poster::Image(img)) = &poster {
        button(Image::<image::Handle>::new(img.clone())).width(width)
    } else {
        button("IMG").width(width)
    }
    .on_press(Message::FilterChanged(Filter::Details(id)))
    .into()
}
pub(crate) fn view_details(
    movie: &TmdbMovie,
    details: Option<&MovieDetails>,
    poster: Option<&Poster>,
    current: Option<&EpisodeDetails>,
) -> Element<'static, Message> {
    let mut poster_row = Row::new();
    if let Some(poster) = poster {
        let Poster::Image(image) = poster;
        poster_row = poster_row
            .push(Image::<image::Handle>::new(image.clone()).width(Length::FillPortion(1)));
    }
    let latest_episode_block: Element<_, _, _> = if let Some(details) = details {
        let latest = details.last_published().unwrap();
        row![
            text("Latest Episode: "),
            column![text(&latest.name), text(latest.episode.as_info_str())]
        ]
        .into()
    } else {
        Space::with_height(Length::FillPortion(1)).into()
    };
    let details_block = column![
        if let Some(current) = current {
            row![
                text("Current Episode: "),
                column![text(&current.name), text(current.episode.as_info_str())]
            ]
        } else {
            row![
                text("Current Episode: "),
                column![text("TITLE"), text("S 2 E 2")]
            ]
        },
        latest_episode_block
    ]
    .width(Length::FillPortion(3));
    poster_row = poster_row.push(details_block);
    column![
        button("back").on_press(Message::FilterChanged(Filter::Bookmarks)),
        row![
            text(&movie.name).size(FONT_SIZE_HEADER),
            text(format!(" [{}]", &movie.original_name)).size(FONT_SIZE_HEADER)
        ],
        poster_row
    ]
    .into()
}
