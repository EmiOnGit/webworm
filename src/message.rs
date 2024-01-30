use iced::font::{self};

use iced::widget::{container, text};
use iced::{alignment, window, Color, Element, Length};

use crate::filter::Filter;
use crate::movie::MovieMessage;
use crate::save::{LoadError, SaveError, SavedState};
use crate::tmdb::RequestType;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    FontLoaded(Result<(), font::Error>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    ExecuteRequest(RequestType),
    RequestResponse(Option<String>, RequestType),
    FilterChanged(Filter),
    MovieMessage(usize, MovieMessage),
    BookmarkMessage(usize, BookmarkMessage),
    TabPressed { shift: bool },
    ToggleFullscreen(window::Mode),
}
pub fn loading_message<'a>() -> Element<'a, Message> {
    container(
        text("Loading...")
            .horizontal_alignment(alignment::Horizontal::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}
pub fn empty_message(message: &str) -> Element<'_, Message> {
    container(
        text(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .style(Color::from([0.7, 0.7, 0.7])),
    )
    .height(200)
    .center_y()
    .into()
}
#[derive(Clone, Debug)]
pub enum BookmarkMessage {
    Remove,
    IncrE,
    IncrS,
    DecrE,
    DecrS,
    LinkInputChanged(String),
    LinkInputSubmit,
    LinkToClipboard,
}
