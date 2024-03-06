use std::iter::once;

use crate::filter::Filter;
use crate::id::{EpisodeId, MovieIndex};
use crate::movie_details::EpisodeDetails;
use crate::save::{LoadError, SavedState};
use crate::state::{GuiState, InputKind, State};
use crate::view;
use iced::alignment::{self, Alignment, Horizontal, Vertical};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, keyed_column, row, scrollable, text, text_input, Space,
};
use iced::window;
use iced::{Application, Element};
use iced::{Color, Command, Length, Subscription};
use once_cell::sync::Lazy;
use tracing::error;

use crate::bookmark::Bookmark;
use crate::message::{empty_message, loading_message, Message, ShiftPressed};

const TITLE_NAME: &str = "Webworm";
static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub(crate) static FONT_SIZE_HEADER: u16 = 30;
pub(crate) static FONT_SIZE: u16 = 22;
static FG_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);
static ERROR_COLOR: Color = Color::from_rgb(1., 0.2, 0.2);

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum App {
    Loading,
    CreateNew(String),
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
                    Message::Loaded(Err(e)) => {
                        *self = match e {
                            LoadError::DeserializationError(s) => App::CreateNew(s),
                            _ => App::Loaded(State::default()),
                        };
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
            App::CreateNew(_error) => {
                match message {
                    Message::CreateNew => *self = App::Loaded(State::default()),
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            App::Loading => loading_message(),
            App::Loaded(State {
                gui:
                    GuiState {
                        input_caches,
                        filter,
                        ..
                    },
                movies,
                movie_details,
                movie_posters,
                episode_details,
                bookmarks,
                ..
            }) => {
                let header = view_header();
                let input = view_input(&input_caches[InputKind::SearchField]);
                let mut control_info = None;
                let body = match filter {
                    Filter::Search => {
                        if movies.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            control_info =
                                Some(format!("{} results found for search", movies.len()));
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
                                .filter(|bookmark| {
                                    bookmark
                                        .movie
                                        .matches_filter(&input_caches[InputKind::SearchField])
                                })
                                .collect();
                            control_info = Some(format!("{} movies left", bookmarks.len()));
                            let chunk_size = 3;
                            let chunks = bookmarks.chunks_exact(chunk_size);
                            let remainder = chunks.remainder();
                            let remainder_row =
                                (0..chunk_size).map(|i| remainder.get(i)).map(|bookmark| {
                                    if let Some(bookmark) = bookmark {
                                        bookmark.card_view(
                                            movie_details.get(&bookmark.movie.id),
                                            movie_posters.get(&bookmark.movie.id),
                                        )
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
                        let current_episode_details = episode_details
                            .get(&EpisodeId(*id, bookmark.current_episode.clone()))
                            .cloned()
                            .or_else(|| {
                                if let Some(details) = details {
                                    let seasonal =
                                        details.as_seasonal_episode(&bookmark.current_episode);
                                    Some(EpisodeDetails {
                                        episode: seasonal,
                                        name: "".into(),
                                        air_date: None,
                                        overview: "".into(),
                                    })
                                } else {
                                    None
                                }
                            });
                        view::view_details(
                            movie,
                            input_caches,
                            details,
                            poster,
                            current_episode_details,
                            bookmark.sync_mode,
                        )
                    }
                };
                let controls = view_controls(control_info, *filter);
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
            App::CreateNew(error) => {
                let create = button("OVERRIDE OLD STATE").on_press(Message::CreateNew);
                container(
                    column![
                        row![
                            text("State loading failed: ")
                                .size(FONT_SIZE)
                                .style(FG_COLOR),
                            text(error).size(FONT_SIZE).style(ERROR_COLOR),
                        ]
                        .spacing(20),
                        create
                    ]
                    .align_items(Alignment::Center)
                    .spacing(20),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
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
fn view_controls(
    control_info: Option<String>,
    current_filter: Filter,
) -> Element<'static, Message> {
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
        text(control_info.unwrap_or_default()).width(Length::Fill),
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

    row![title].into()
}
fn view_input(input: &str) -> Element<'static, Message> {
    text_input("Search", input)
        .id(INPUT_ID.clone())
        .on_input(|input| Message::InputChanged(InputKind::SearchField, input))
        .on_submit(Message::InputSubmit(InputKind::SearchField))
        .padding(15)
        .size(FONT_SIZE)
        .into()
}
