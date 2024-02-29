use std::iter::once;

use crate::filter::Filter;
use crate::id::{EpisodeId, MovieIndex};
use crate::movie::TmdbMovie;
use crate::save::SavedState;
use crate::state::State;
use crate::view;
use iced::alignment::{self, Alignment};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, keyed_column, row, scrollable, text, text_input, Space, Text,
};
use iced::window::{self};
use iced::{Application, Element};
use iced::{Color, Command, Length, Subscription};
use once_cell::sync::Lazy;
use tracing::error;

use crate::bookmark::Bookmark;
use crate::message::{empty_message, loading_message, Message, ShiftPressed};

const TITLE_NAME: &str = "Webworm";
static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static INPUT_LINK_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static FONT_SIZE_HEADER: u16 = 30;
pub static FONT_SIZE: u16 = 18;
static FG_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum App {
    Loading,
    Loaded(State),
}
impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App::Loading,
            Command::batch(vec![Command::perform(SavedState::load(), Message::Loaded)]),
        )
    }

    fn title(&self) -> String {
        TITLE_NAME.to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            App::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => return self.as_loaded(state),
                    Message::Loaded(Err(_)) => {
                        error!("Something went wrong with loading the app state. Default configuration is used");
                        *self = App::Loaded(State::default());
                    }
                    _ => {
                        error!("Received message {message:?} in loading phase")
                    }
                }

                text_input::focus(INPUT_ID.clone())
            }
            App::Loaded(state) => {
                let update = state.update_state(message);
                let saved = update.has_just_saved();
                let save = state.save(saved);
                Command::batch(vec![update.command(), save])
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            App::Loading => loading_message(),
            App::Loaded(State {
                input_value,
                filter,
                movies,
                movie_details,
                movie_posters,
                episode_details,
                links,
                bookmarks,
                ..
            }) => {
                let header = view_header();
                let input = view_input(input_value);
                let controls = view_controls(movies, *filter);
                let body = match filter {
                    Filter::Search => {
                        if movies.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            keyed_column(
                                movies
                                    .iter()
                                    .map(|task| (task.id, task.view(movie_posters.get(&task.id)))),
                            )
                            .spacing(10)
                            .into()
                        }
                    }
                    Filter::Bookmarks | Filter::Completed => {
                        if bookmarks.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            let bookmarks: Vec<&Bookmark> = bookmarks
                                .iter()
                                .filter(|bookmark| {
                                    if Filter::Completed == *filter {
                                        bookmark.finished
                                    } else {
                                        !bookmark.finished
                                    }
                                })
                                .filter(|bookmark| bookmark.movie.matches_filter(&input_value))
                                .collect();
                            let chunk_size = 3;
                            let chunks = bookmarks.chunks_exact(chunk_size);
                            let remainder = chunks.remainder();
                            let remainder_row =
                                (0..chunk_size).map(|i| remainder.get(i)).map(|bookmark| {
                                    if let Some(bookmark) = bookmark {
                                        bookmark
                                            .card_view(
                                                movie_details.get(&bookmark.movie.id),
                                                links.get(&bookmark.movie.id).unwrap(),
                                                movie_posters.get(&bookmark.movie.id),
                                            )
                                            .into()
                                    } else {
                                        Space::with_width(Length::Fill).into()
                                    }
                                });
                            column(
                                chunks
                                    .map(|bookmarks| {
                                        bookmarks.iter().map(|bookmark| {
                                            bookmark.card_view(
                                                movie_details.get(&bookmark.movie.id),
                                                links.get(&bookmark.movie.id).unwrap(),
                                                movie_posters.get(&bookmark.movie.id),
                                            )
                                        })
                                    })
                                    .map(|it| row(it).spacing(50).into())
                                    .chain(once(row(remainder_row).into())),
                            )
                            .spacing(50)
                            .into()
                        }
                    }
                    Filter::Details(id) => {
                        let bookmark = bookmarks
                            .with_id(*id)
                            .expect("tried to show details for bookmark that does not exist");
                        let movie = &bookmark.movie;
                        let details = movie_details.get(id);
                        let poster = movie_posters.get(id);
                        let current_episode_details =
                            episode_details.get(&EpisodeId(*id, bookmark.current_episode.clone()));
                        view::view_details(movie, details, poster, current_episode_details)
                    }
                };
                let content = match filter {
                    Filter::Bookmarks | Filter::Search | Filter::Completed => {
                        column![header, input, controls, body]
                    }
                    Filter::Details(_) => column![header, body],
                }
                .spacing(20)
                .max_width(1000);

                scrollable(container(content).padding(40).center_x()).into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;
        let on_press = keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::Tab, _) => Some(Message::TabPressed),
                (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
                }
                (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Windowed))
                }
                (key::Named::Shift, _) => Some(Message::ShiftPressed(ShiftPressed::True)),
                _ => None,
            }
        });
        let on_release = keyboard::on_key_release(|key, _modifier| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };
            if key == key::Named::Shift {
                Some(Message::ShiftPressed(ShiftPressed::False))
            } else {
                None
            }
        });
        Subscription::batch([on_press, on_release])
    }
}
fn view_controls(movies: &[TmdbMovie], current_filter: Filter) -> Element<Message> {
    let movies_left = movies.len();

    let filter_button = |label, filter, current_filter| {
        let label = text(label);

        let button = button(label).style(if filter == current_filter {
            theme::Button::Primary
        } else {
            theme::Button::Text
        });

        button.on_press(Message::FilterChanged(filter)).padding(8)
    };

    row![
        text(format!(
            "{movies_left} {} left",
            if movies_left == 1 { "task" } else { "movies" }
        ))
        .width(Length::Fill),
        row![
            filter_button("Bookmarks", Filter::Bookmarks, current_filter),
            filter_button("Search", Filter::Search, current_filter),
            filter_button("Completed", Filter::Completed, current_filter,),
        ]
        .width(Length::Shrink)
        .spacing(10)
    ]
    .spacing(20)
    .align_items(Alignment::Center)
    .into()
}
fn view_header() -> Element<'static, Message> {
    let title = text("Webworm")
        .size(FONT_SIZE_HEADER)
        .style(FG_COLOR)
        .horizontal_alignment(alignment::Horizontal::Left);

    let settings = icon('⚙')
        .width(Length::Fill)
        .size(FONT_SIZE_HEADER)
        .style(FG_COLOR)
        .horizontal_alignment(alignment::Horizontal::Right);
    row![title, settings].into()
}
fn view_input(input: &str) -> Element<'static, Message> {
    text_input("Search", input)
        .id(INPUT_ID.clone())
        .on_input(Message::InputChanged)
        .on_submit(Message::InputSubmit(input.to_string()))
        .padding(15)
        .size(FONT_SIZE)
        .into()
}
pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        // .font(crate::gui::ICON_FONT)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}
