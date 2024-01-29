mod message;

use crate::filter::Filter;
use crate::save::SavedState;
use crate::task::{icon, TaskMessage, TmdbMovie};
use crate::tmdb::{queue_tv_series, request, TmdbConfig};
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

use self::message::{empty_message, loading_message, Message};

const TITLE_NAME: &str = "Webworm";
pub const ICON_FONT: Font = Font::with_name("Noto Color Emoji");
const ICON_FONT_BYTES: &[u8] = include_bytes!("../../assets/NotoColorEmoji.ttf");
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
                            input_value: state.input_value,
                            filter: state.filter,
                            movies: state.tasks,
                            tmdb_config: state.tmdb_config,
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
                                .expect("tmdb config is not loaded");
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
                        println!("RESPONSE:");
                        println!("{text:?}");
                        Command::none()
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;

                        Command::none()
                    }
                    Message::TaskMessage(i, TaskMessage::Delete) => {
                        state.movies.remove(i);

                        Command::none()
                    }
                    Message::TaskMessage(i, task_message) => {
                        if let Some(task) = state.movies.get_mut(i) {
                            let should_focus = matches!(task_message, TaskMessage::Edit);

                            task.update(task_message);

                            if should_focus {
                                let id = TmdbMovie::text_input_id(i);
                                Command::batch(vec![
                                    text_input::focus(id.clone()),
                                    text_input::select_all(id),
                                ])
                            } else {
                                Command::none()
                            }
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
                            input_value: state.input_value.clone(),
                            filter: state.filter,
                            tasks: state.movies.clone(),
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
                ..
            }) => {
                let title = text("Webworm")
                    .size(FONT_SIZE_HEADER)
                    .style(FG_COLOR)
                    .horizontal_alignment(alignment::Horizontal::Left);
                let layer = text("Layer")
                    // .width(Length::Fill)
                    .size(FONT_SIZE_HEADER)
                    .style(FG_COLOR)
                    .horizontal_alignment(alignment::Horizontal::Center);

                // let settings = icon('\u{0F00DE}')
                let settings = icon('⚙')
                    .width(Length::Fill)
                    .size(FONT_SIZE_HEADER)
                    .style(FG_COLOR)
                    .horizontal_alignment(alignment::Horizontal::Right);

                let input = text_input("What needs to be done?", input_value)
                    .id(INPUT_ID.clone())
                    .on_input(Message::InputChanged)
                    .on_submit(Message::ExecuteQuery)
                    .padding(15)
                    .size(FONT_SIZE);

                let controls = view_controls(tasks, *filter);
                let filtered_tasks = tasks.iter().filter(|task| filter.matches(task));

                let tasks: Element<_> = if filtered_tasks.count() > 0 {
                    keyed_column(
                        tasks
                            .iter()
                            .enumerate()
                            .filter(|(_, task)| filter.matches(task))
                            .map(|(i, task)| {
                                (
                                    task.id,
                                    task.view(i)
                                        .map(move |message| Message::TaskMessage(i, message)),
                                )
                            }),
                    )
                    .spacing(10)
                    .into()
                } else {
                    empty_message(match filter {
                        Filter::All => "You have not created a task yet...",
                        Filter::Active => "All your tasks are done! :D",
                        Filter::Completed => "You have not completed a task yet...",
                    })
                };
                let row = row![title, layer, settings];
                let content = column![row, input, controls, tasks]
                    .spacing(20)
                    .max_width(800);

                scrollable(container(content).padding(40).center_x()).into()
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
    let tasks_left = tasks.iter().filter(|task| !task.is_completed()).count();

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
            filter_button("All", Filter::All, current_filter),
            filter_button("Active", Filter::Active, current_filter),
            filter_button("Completed", Filter::Completed, current_filter,),
        ]
        .width(Length::Shrink)
        .spacing(10)
    ]
    .spacing(20)
    .align_items(Alignment::Center)
    .into()
}
