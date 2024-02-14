use std::collections::HashMap;

use crate::filter::Filter;
use crate::movie::{icon, MovieMessage, TmdbMovie};
use crate::movie_details::MovieDetails;
use crate::save::{load_poster, SavedState};
use crate::tmdb::{send_request, RequestType, TmdbConfig, TmdbResponse};
use iced::alignment::{self, Alignment};
use iced::font::{self, Font};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    self, button, column, container, keyed_column, row, scrollable, text, text_input,
};
use iced::window::{self};
use iced::{Application, Element};
use iced::{Color, Command, Length, Subscription};
use once_cell::sync::Lazy;
use serde_json::Value;
use tracing::{error, warn};

use crate::bookmark::{Bookmark, Poster};
use crate::message::{empty_message, loading_message, BookmarkMessage, Message};

const TITLE_NAME: &str = "Webworm";
pub const ICON_FONT: Font = Font::with_name("Noto Color Emoji");
const ICON_FONT_BYTES: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");
static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static INPUT_LINK_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static FONT_SIZE_HEADER: u16 = 30;
pub static FONT_SIZE: u16 = 18;
static FG_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);

#[derive(Debug)]
pub enum App {
    Loading,
    Loaded(State),
}
#[derive(Debug, Default)]
pub struct State {
    input_value: String,
    filter: Filter,
    movies: Vec<TmdbMovie>,
    movie_details: HashMap<usize, MovieDetails>,
    bookmarks: Vec<Bookmark>,
    dirty: bool,
    saving: bool,
    tmdb_config: Option<TmdbConfig>,
    debug: DebugState,
}
#[derive(Default, Debug, PartialEq)]
pub enum DebugState {
    Debug,
    #[default]
    Release,
}
impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App::Loading,
            Command::batch(vec![
                font::load(ICON_FONT_BYTES).map(Message::FontLoaded),
                Command::perform(SavedState::load(), Message::Loaded),
            ]),
        )
    }

    fn title(&self) -> String {
        TITLE_NAME.to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            App::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        // set state to be loaded
                        *self = App::Loaded(State {
                            tmdb_config: state.tmdb_config,
                            bookmarks: state.bookmarks.clone(),
                            ..State::default()
                        });
                        // load new data for the bookmarks
                        let iter_load_details = state
                            .bookmarks
                            .iter()
                            .map(|bookmark| RequestType::TvDetails { id: bookmark.id })
                            .map(|req| {
                                Command::perform(async { Ok(()) }, |_: Result<(), ()>| {
                                    Message::ExecuteRequest(req)
                                })
                            });
                        let iter_load_posters = state
                            .bookmarks
                            .iter()
                            .map(|bookmark| RequestType::Poster {
                                id: bookmark.id,
                                path: bookmark.poster_path.clone(),
                            })
                            .map(|req| {
                                Command::perform(async { Ok(()) }, |_: Result<(), ()>| {
                                    Message::ExecuteRequest(req)
                                })
                            });

                        return Command::batch(iter_load_details.chain(iter_load_posters));
                    }
                    Message::Loaded(Err(_)) => {
                        error!("Something went wrong with loading the app state. Default configuration is used");
                        *self = App::Loaded(State::default());
                    }
                    Message::FontLoaded(_) => {}
                    _ => {
                        error!("Received message {message:?} in loading phase")
                    }
                }

                text_input::focus(INPUT_ID.clone())
            }
            App::Loaded(state) => {
                let mut saved = false;

                let command = match message {
                    Message::InputChanged(value) => {
                        state.input_value = value;

                        Command::none()
                    }
                    Message::ExecuteRequest(request) => {
                        if let RequestType::TvSearch { .. } = request {
                            state.input_value.clear();
                        }
                        let config = state
                            .tmdb_config
                            .clone()
                            .expect("TMDB config is not loaded");
                        if let RequestType::Poster { id, path } = request {
                            Command::perform(load_poster(id, path.clone(), config), move |data| {
                                Message::RequestPoster(data.ok(), id)
                            })
                        } else {
                            Command::perform(send_request(config, request.clone()), |data| {
                                Message::RequestResponse(data.ok(), request)
                            })
                        }
                    }
                    Message::RequestPoster(handle, id) => {
                        if let Some(handle) = handle {
                            let bookmark = state.bookmarks.iter_mut().find(|b| b.id == id).unwrap();
                            bookmark.poster = Poster::Image(handle);
                        }
                        Command::none()
                    }
                    Message::RequestResponse(text, query) => {
                        if let Some(text) = text {
                            match query {
                                RequestType::TvSearch { .. } => {
                                    let mut response: TmdbResponse =
                                        serde_json::from_str(&text).unwrap();
                                    state.movies = response.movies(&state.bookmarks).clone();
                                }
                                RequestType::TvDetails { .. } => {
                                    let Ok(response) = serde_json::from_str::<MovieDetails>(&text)
                                    else {
                                        error!("failed with:");
                                        let res: Value = serde_json::from_str(&text).unwrap();
                                        let pretty = serde_json::to_string_pretty(&res).unwrap();
                                        println!("{pretty}");
                                        panic!()
                                    };
                                    if state.debug == DebugState::Debug {
                                        let res: Value = serde_json::from_str(&text).unwrap();
                                        let pretty = serde_json::to_string_pretty(&res).unwrap();
                                        println!("{pretty}");
                                    }
                                    state.movie_details.insert(response.id, response);
                                }
                                RequestType::Poster { id: _, path: _ } => {}
                            }
                        }
                        Command::none()
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;
                        Command::none()
                    }
                    Message::MovieMessage(i, MovieMessage::ToggleBookmark) => {
                        if let Some(movie) = state.movies.get_mut(i) {
                            movie.update(MovieMessage::ToggleBookmark);
                            let movie: &TmdbMovie = movie;
                            if let Some(index) =
                                state.bookmarks.iter().position(|b| b.id == movie.id)
                            {
                                state.bookmarks.remove(index);
                            } else {
                                state.bookmarks.push(Bookmark::from(movie));
                            }
                        }
                        Command::none()
                    }
                    Message::BookmarkMessage(i, BookmarkMessage::Remove) => {
                        if i < state.bookmarks.len() {
                            state.bookmarks.remove(i);
                        } else {
                            warn!("tried to remove bookmark at place {}, but there are only {} bookmarks", i + 1, state.bookmarks.len() + 1)
                        }
                        Command::none()
                    }
                    Message::BookmarkMessage(i, message) => {
                        if let Some(bookmark) = state.bookmarks.get_mut(i) {
                            bookmark.apply(message)
                        } else {
                            Command::none()
                        }
                    }
                    Message::Saved(_) => {
                        state.saving = false;
                        saved = true;

                        Command::none()
                    }
                    Message::TabPressed { shift } => {
                        if shift {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    Message::ToggleFullscreen(mode) => window::change_mode(window::Id::MAIN, mode),
                    Message::Loaded(_) => {
                        error!("Loaded the app state even though it should already was loaded");
                        Command::none()
                    }
                    Message::FontLoaded(_) => {
                        error!("Loaded font after loading state.");
                        Command::none()
                    }
                };

                if !saved {
                    state.dirty = true;
                }

                let save = if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;
                    Command::perform(
                        SavedState {
                            bookmarks: state.bookmarks.clone(),
                            // We ignore it anyway since we save it in a text file
                            tmdb_config: None,
                        }
                        .save(),
                        Message::Saved,
                    )
                } else {
                    Command::none()
                };

                Command::batch(vec![command, save])
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            App::Loading => loading_message(),
            App::Loaded(State {
                input_value,
                filter,
                movies: tasks,
                movie_details,
                bookmarks,
                ..
            }) => {
                let header = view_header();
                let input = view_input(input_value);
                let controls = view_controls(tasks, *filter);
                let body = match filter {
                    Filter::Search => {
                        if tasks.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            keyed_column(
                                tasks
                                    .iter()
                                    .enumerate()
                                    .map(|(i, task)| (task.id, task.view(i))),
                            )
                            .spacing(10)
                            .into()
                        }
                    }
                    Filter::Bookmarks | Filter::Completed => {
                        if bookmarks.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            keyed_column(
                                bookmarks
                                    .iter()
                                    .filter(|bookmark| {
                                        if Filter::Completed == *filter {
                                            bookmark.finished
                                        } else {
                                            !bookmark.finished
                                        }
                                    })
                                    .map(|bookmark| (bookmark, movie_details.get(&bookmark.id)))
                                    .enumerate()
                                    .map(|(i, (bookmark, details))| {
                                        (
                                            bookmark.id,
                                            bookmark.view(i, details).map(move |message| {
                                                Message::BookmarkMessage(i, message)
                                            }),
                                        )
                                    }),
                            )
                            .spacing(10)
                            .into()
                        }
                    }
                };

                let content = column![header, input, controls, body]
                    .spacing(20)
                    .max_width(800);

                return scrollable(container(content).padding(40).center_x()).into();
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::Tab, _) => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
                }
                (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Windowed))
                }
                _ => None,
            }
        })
    }
}
fn view_controls(tasks: &[TmdbMovie], current_filter: Filter) -> Element<Message> {
    let tasks_left = tasks.len();

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
            "{tasks_left} {} left",
            if tasks_left == 1 { "task" } else { "tasks" }
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
        .on_submit(Message::ExecuteRequest(RequestType::TvSearch {
            query: input.to_owned(),
        }))
        .padding(15)
        .size(FONT_SIZE)
        .into()
}