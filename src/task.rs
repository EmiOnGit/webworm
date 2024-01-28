use iced::theme::{self};
use iced::widget::{button, checkbox, row, text, text_input, Text};
use iced::{alignment, Length};
use iced::{Alignment, Element};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    description: String,
    completed: bool,

    #[serde(skip)]
    state: TaskState,
}
impl Task {
    pub fn is_completed(&self) -> bool {
        self.completed
    }
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle,
    Editing,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl Task {
    pub fn text_input_id(i: usize) -> text_input::Id {
        text_input::Id::new(format!("task-{i}"))
    }

    pub fn new(description: String) -> Self {
        Task {
            id: Uuid::new_v4(),
            description,
            completed: false,
            state: TaskState::Idle,
        }
    }

    pub fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;
            }
            TaskMessage::Edit => {
                self.state = TaskState::Editing;
            }
            TaskMessage::DescriptionEdited(new_description) => {
                self.description = new_description;
            }
            TaskMessage::FinishEdition => {
                if !self.description.is_empty() {
                    self.state = TaskState::Idle;
                }
            }
            TaskMessage::Delete => {}
        }
    }

    pub fn view(&self, i: usize) -> Element<TaskMessage> {
        match &self.state {
            TaskState::Idle => {
                let checkbox = checkbox(&self.description, self.completed, TaskMessage::Completed)
                    .width(Length::Fill)
                    .text_shaping(text::Shaping::Advanced);

                row![
                    checkbox,
                    button(icon('✍'))
                        .on_press(TaskMessage::Edit)
                        .padding(10)
                        .style(theme::Button::Text),
                ]
                .spacing(20)
                .align_items(Alignment::Center)
                .into()
            }
            TaskState::Editing => {
                let text_input = text_input("Describe your task...", &self.description)
                    .id(Self::text_input_id(i))
                    .on_input(TaskMessage::DescriptionEdited)
                    .on_submit(TaskMessage::FinishEdition)
                    .padding(10);

                row![
                    text_input,
                    button(
                        row![icon('❌'), "Delete"]
                            .spacing(10)
                            .align_items(Alignment::Center)
                    )
                    .on_press(TaskMessage::Delete)
                    .padding(10)
                    .style(theme::Button::Destructive)
                ]
                .spacing(20)
                .align_items(Alignment::Center)
                .into()
            }
        }
    }
}
pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(crate::gui::ICON_FONT)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}
