use iced::alignment::{Horizontal, Vertical};
use iced::theme::{self};
use iced::widget::{button, column, image, row, text, text_input};
use iced::Length;
use iced::{Alignment, Element};

use crate::bookmark::{Bookmark, BookmarkLinkBox, Poster};
use crate::gui::{icon, FONT_SIZE, FONT_SIZE_HEADER, INPUT_LINK_ID};
use crate::message::{BookmarkMessage, Message};
use crate::movie::{MovieMessage, TmdbMovie};
use crate::movie_details::{Episode, MovieDetails};

impl Bookmark {
    pub fn view<'a>(
        &'a self,
        _i: usize,
        details: Option<&MovieDetails>,
        poster: Option<&'a Poster>,
    ) -> Element<BookmarkMessage> {
        let body = row![
            picture_view(poster, Length::FillPortion(3)),
            column![
                row![column![
                    text(self.movie.name.as_str())
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::FillPortion(1))
                        .size(FONT_SIZE_HEADER)
                        .style(theme::Text::Default),
                    row![
                        if let Some(details) = &details {
                            text(format!(
                                "LATEST: {}",
                                Into::<Episode>::into(details.last_published()).as_info_str()
                            ))
                            .width(Length::FillPortion(1))
                        } else {
                            text("details not loaded").width(Length::FillPortion(1))
                        },
                        text(format!("PROGRESS: {}", self.current_episode.as_info_str()))
                            .width(Length::FillPortion(1))
                            .horizontal_alignment(Horizontal::Right)
                    ]
                    .align_items(Alignment::Center)
                ],],
                self.link_view(details)
            ]
            .width(Length::FillPortion(5)),
            button(if self.show_details { "↑" } else { "↓" })
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
                                text(format!("{}", self.current_episode.as_info_str()))
                                    .vertical_alignment(Vertical::Bottom),
                                button("↓")
                                    .style(theme::Button::Secondary)
                                    .padding(10)
                                    .on_press(BookmarkMessage::DecrE(details.cloned()))
                            ])
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                            iced::widget::container(
                                button("---")
                                    .on_press(BookmarkMessage::RemoveLink)
                                    .padding(30)
                                    .style(theme::Button::Secondary)
                            )
                            .align_x(Horizontal::Right),
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
                .width(Length::Fill)
                .padding(15)
                .size(FONT_SIZE)
                .into(),
        }
    }
}
impl TmdbMovie {
    pub fn view<'a>(&'a self, i: usize, poster: Option<&'a Poster>) -> Element<Message> {
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
                picture_view(poster, Length::FillPortion(2))
                    .map(move |bookmark_mess| Message::BookmarkMessage(i, bookmark_mess)),
                info_column.width(Length::FillPortion(2)),
                text(description + "...")
                    .width(Length::FillPortion(6))
                    .style(theme::Text::Default),
                button(icon('✍'))
                    .on_press(Message::MovieMessage(i, MovieMessage::ToggleBookmark))
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
fn picture_view(poster: Option<&Poster>, width: Length) -> Element<BookmarkMessage> {
    if let Some(Poster::Image(img)) = &poster {
        image::viewer(img.clone())
            .width(Length::Fixed(500.))
            .height(Length::Fixed(200.))
            .into()
    } else {
        iced::widget::text("IMG").width(width).into()
    }
}
