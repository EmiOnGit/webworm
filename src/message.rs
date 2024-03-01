use iced::widget::image::Handle;
use iced::widget::{container, text};
use iced::{alignment, window, Color, Element, Length};

use crate::filter::Filter;
use crate::id::MovieId;
use crate::movie_details::MovieDetails;
use crate::save::{LoadError, SaveError, SavedState};
use crate::state::InputKind;
use crate::tmdb::RequestType;

#[derive(Debug, Clone)]
pub enum Message {
    // Still loading
    Loaded(Result<SavedState, LoadError>),
    // Finished loading
    Saved(Result<(), SaveError>),
    InputChanged(InputKind, String),
    InputSubmit(InputKind),
    ExecuteRequest(RequestType),
    RequestResponse(Option<String>, RequestType),
    RequestPoster(MovieId, Option<Handle>),
    FilterChanged(Filter),
    AddBookmark(MovieId),
    RemoveBookmark(MovieId),
    BookmarkMessage(MovieId, BookmarkMessage),
    LinkMessage(MovieId, LinkMessage),
    TabPressed,
    ShiftPressed(ShiftPressed),
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
    IncrE(Option<MovieDetails>),
    DecrE(Option<MovieDetails>),
    SetE(String, Option<MovieDetails>),
    SetS(String, Option<MovieDetails>),
    ToggleSync,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ShiftPressed {
    True,
    #[default]
    False,
    Unknown,
}
#[derive(Clone, Debug)]
pub enum LinkMessage {
    LinkToClipboard(Option<MovieDetails>, ShiftPressed),
}
