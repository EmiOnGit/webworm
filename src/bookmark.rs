use iced::theme::{self};
use iced::widget::{button, column, row, text};
use iced::Length;
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};

use crate::message::BookmarkMessage;
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
    pub fn apply(&mut self, action: BookmarkMessage) {
        match action {
            BookmarkMessage::IncrE => self.current_episode += 1,
            BookmarkMessage::IncrS => self.current_season += 1,
            BookmarkMessage::DecrE => self.current_episode = (self.current_episode - 1).max(1),
            BookmarkMessage::DecrS => self.current_season = (self.current_season - 1).max(1),
        }
    }
    pub fn view(&self, _i: usize) -> Element<BookmarkMessage> {
        let info_column = column![text(self.name.as_str()).style(theme::Text::Default),];
        iced::widget::container(
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
                    button("incr E").on_press(BookmarkMessage::IncrE),
                    button("incr S").on_press(BookmarkMessage::IncrS),
                ],
                column![
                    button("decr E").on_press(BookmarkMessage::DecrE),
                    button("decr S").on_press(BookmarkMessage::DecrS),
                ],
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
