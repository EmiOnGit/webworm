use iced::theme::{self};
use iced::widget::{button, column, row, text, Text};
use iced::{alignment, Length};
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};

use crate::bookmark::Bookmark;
use crate::message::Message;
use crate::tmdb::RequestType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TmdbMovie {
    pub id: usize,
    pub genre_ids: Vec<usize>,
    pub overview: String,
    pub vote_average: f32,
    original_name: String,
    pub name: String,
    pub popularity: f32,
    pub poster_path: String,

    #[serde(skip)]
    pub is_bookmark: bool,
}

#[derive(Debug, Clone)]
pub enum MovieMessage {
    ToggleBookmark,
}

impl TmdbMovie {
    fn rating(&self) -> u8 {
        (self.vote_average * 10.) as u8
    }
    pub fn set_bookmark(&mut self, bookmarks: &[Bookmark]) {
        self.is_bookmark = bookmarks.iter().any(|bookmark| bookmark.id == self.id);
    }

    pub fn update(&mut self, message: MovieMessage) {
        match message {
            MovieMessage::ToggleBookmark => {
                self.is_bookmark = !self.is_bookmark;
            }
        }
    }

    pub fn view(&self, i: usize) -> Element<Message> {
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
                info_column.width(Length::FillPortion(2)),
                text(description + "...")
                    .width(Length::FillPortion(6))
                    .style(theme::Text::Default),
                button(icon('✍'))
                    .on_press(Message::MovieMessage(i, MovieMessage::ToggleBookmark))
                    .padding(10)
                    .width(Length::FillPortion(1))
                    .style(theme::Button::Text),
                button(text("details"))
                    .on_press(Message::ExecuteRequest(RequestType::TvDetails {
                        id: self.id
                    }))
                    .padding(10)
                    .width(Length::FillPortion(1))
                    .style(theme::Button::Text),
            ]
            .spacing(20)
            .align_items(Alignment::Start),
        )
        .style(if self.is_bookmark {
            theme::Container::Box
        } else {
            theme::Container::Transparent
        })
        .width(Length::Fill)
        .padding(20.)
        .into()
    }
}
pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(crate::gui::ICON_FONT)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}