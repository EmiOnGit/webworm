use crate::filter::Filter;
use crate::movie::{MovieMessage, TmdbMovie};
use crate::movie_details::{Episode, MovieDetails};
use crate::save::{load_poster, SavedState};
use crate::state::{DebugState, State};
use crate::tmdb::{send_request, RequestType, TmdbResponse};
use iced::alignment::{self, Alignment};
use iced::font::{self, Font};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    self, button, column, container, keyed_column, row, scrollable, text, text_input, Text,
};
use iced::window::{self};
use iced::{Application, Element};
use iced::{Color, Command, Length, Subscription};
use once_cell::sync::Lazy;
use serde_json::Value;
use tracing::{error, info, warn};

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
                    Message::Loaded(Ok(state)) => return self.as_loaded(state),
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
                            println!("insert {id}");
                            state.movie_posters.insert(id, Poster::Image(handle));
                        }
                        Command::none()
                    }
                    Message::RequestResponse(text, query) => {
                        let Some(text) = text else {
                            return Command::none();
                        };
                        match query {
                            RequestType::TvSearch { .. } => {
                                let response: serde_json::Result<TmdbResponse> =
                                    serde_json::from_str(&text);
                                match response {
                                    Ok(response) => state.movies = response.results,

                                    Err(e) => {
                                        error!("{e:?}");
                                        return Command::none();
                                    }
                                }
                                let mut cmds = Vec::new();
                                for movie in &state.movies {
                                    let id = movie.id;
                                    let cmd: Command<Message> =
                                        Command::perform(async { () }, move |_: ()| {
                                            Message::ExecuteRequest(RequestType::TvDetails { id })
                                        });
                                    cmds.push(cmd);
                                    if let Some(path) = movie.poster_path.clone() {
                                        let cmd: Command<Message> =
                                            Command::perform(async { () }, move |_: ()| {
                                                Message::ExecuteRequest(RequestType::Poster {
                                                    id,
                                                    path,
                                                })
                                            });
                                        cmds.push(cmd);
                                    }
                                }
                                Command::batch(cmds)
                            }
                            RequestType::TvDetails { .. } => {
                                let Ok(mut response) = serde_json::from_str::<MovieDetails>(&text)
                                else {
                                    error!("failed reading tv details with:");
                                    let res: Value = serde_json::from_str(&text).unwrap();
                                    let pretty = serde_json::to_string_pretty(&res).unwrap();
                                    error!("{pretty}");
                                    panic!()
                                };
                                if state.debug == DebugState::Debug {
                                    let res: Value = serde_json::from_str(&text).unwrap();
                                    let pretty = serde_json::to_string_pretty(&res).unwrap();
                                    info!("{pretty}");
                                }
                                response.fix_episode_formats();
                                if let Some(bookmark) = state
                                    .bookmarks
                                    .iter_mut()
                                    .find(|bookmark| bookmark.movie.id == response.id)
                                {
                                    if let Episode::Total(e) = &bookmark.current_episode {
                                        bookmark.current_episode =
                                            response.as_seasonal_episode(&e).into();
                                    }
                                }
                                state.movie_details.insert(response.id, response);
                                Command::none()
                            }
                            RequestType::Poster { id: _, path: _ } => Command::none(),
                        }
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;
                        Command::none()
                    }
                    Message::MovieMessage(i, MovieMessage::ToggleBookmark) => {
                        if let Some(movie) = state.movies.get_mut(i) {
                            if let Some(index) =
                                state.bookmarks.iter().position(|b| b.movie.id == movie.id)
                            {
                                state.bookmarks.remove(index);
                            } else {
                                state.bookmarks.push(Bookmark::from(&*movie));
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
                let save = state.save(saved);

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
                movies,
                movie_details,
                movie_posters,
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
                            keyed_column(movies.iter().enumerate().map(|(i, task)| {
                                (task.id, task.view(i, movie_posters.get(&task.id)))
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
                                    .map(|bookmark| {
                                        (bookmark, movie_details.get(&bookmark.movie.id))
                                    })
                                    .enumerate()
                                    .map(|(i, (bookmark, details))| {
                                        (
                                            bookmark.movie.id,
                                            bookmark
                                                .view(
                                                    i,
                                                    details,
                                                    movie_posters.get(&bookmark.movie.id),
                                                )
                                                .map(move |message| {
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
        .on_submit(Message::ExecuteRequest(RequestType::TvSearch {
            query: input.to_owned(),
        }))
        .padding(15)
        .size(FONT_SIZE)
        .into()
}
pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(crate::gui::ICON_FONT)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}
