use crate::filter::Filter;
use crate::movie::{icon, MovieMessage, TmdbMovie};
use crate::save::SavedState;
use crate::tmdb::{queue_tv_series, TmdbConfig, TmdbResponse};
use iced::alignment::{self, Alignment};
use iced::font::{self, Font};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    self, button, column, container, keyed_column, row, scrollable, text, text_input,
};
use iced::window;
use iced::{Application, Element};
use iced::{Color, Command, Length, Subscription};
use once_cell::sync::Lazy;

use crate::bookmark::Bookmark;
use crate::message::{empty_message, loading_message, Message};
const TITLE_NAME: &str = "Webworm";
pub const ICON_FONT: Font = Font::with_name("Noto Color Emoji");
const ICON_FONT_BYTES: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");
static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static FONT_SIZE_HEADER: u16 = 30;
static FONT_SIZE: u16 = 18;
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
    bookmarks: Vec<Bookmark>,
    dirty: bool,
    saving: bool,
    tmdb_config: Option<TmdbConfig>,
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
                        *self = App::Loaded(State {
                            movies: state.movies,
                            tmdb_config: state.tmdb_config,
                            bookmarks: state.bookmarks,
                            ..State::default()
                        });
                    }
                    Message::Loaded(Err(_)) => {
                        *self = App::Loaded(State::default());
                    }
                    _ => {}
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
                    Message::ExecuteQuery => {
                        if !state.input_value.is_empty() {
                            let config = state
                                .tmdb_config
                                .clone()
                                .expect("TMDB config is not loaded");
                            let query = state.input_value.clone();
                            state.input_value.clear();
                            Command::perform(queue_tv_series(config, query), |data| {
                                Message::QueryResponse(data.ok())
                            })
                        } else {
                            Command::none()
                        }
                    }
                    Message::QueryResponse(text) => {
                        if let Some(text) = text {
                            let mut response: TmdbResponse = serde_json::from_str(&text).unwrap();

                            state.movies = response.movies(&state.bookmarks).clone();
                        }
                        Command::none()
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;

                        Command::none()
                    }
                    Message::TaskMessage(i, MovieMessage::ToggleBookmark) => {
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
                    _ => Command::none(),
                };

                if !saved {
                    state.dirty = true;
                }

                let save = if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;
                    Command::perform(
                        SavedState {
                            movies: state.movies.clone(),
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
                bookmarks,
                ..
            }) => {
                let input = view_input(input_value);
                let controls = view_controls(tasks, *filter);
                let header = view_header();
                let body = match filter {
                    Filter::Search => {
                        if tasks.is_empty() {
                            empty_message(filter.empty_message())
                        } else {
                            keyed_column(tasks.iter().enumerate().map(|(i, task)| {
                                (
                                    task.id,
                                    task.view(i)
                                        .map(move |message| Message::TaskMessage(i, message)),
                                )
                            }))
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
                                    .enumerate()
                                    .map(|(i, bookmark)| {
                                        (
                                            bookmark.id,
                                            bookmark.view(i).map(move |message| {
                                                Message::TaskMessage(i, message)
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
        .on_submit(Message::ExecuteQuery)
        .padding(15)
        .size(FONT_SIZE)
        .into()
}
