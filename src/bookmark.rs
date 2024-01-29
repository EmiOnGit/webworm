use iced::theme::{self};
use iced::widget::{button, column, row, text};
use iced::Length;
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};

use crate::movie::{MovieMessage, TmdbMovie};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    name: String,
    current_episode: usize,
    current_season: usize,
    latest_season: usize,
    latest_episode: usize,
    pub finished: bool,
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
        }
    }
}
impl Bookmark {
    pub fn view(&self, _i: usize) -> Element<MovieMessage> {
        let info_column = column![text(self.name.as_str()).style(theme::Text::Default),];
        iced::widget::container(
            row![
                info_column,
                button(self.name.as_str()).style(theme::Button::Text),
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .style(theme::Container::Box)
        .width(Length::Fill)
        .padding(20.)
        .into()
    }
}
