use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::state::EntryBuilder;
use crate::{messages::Message, state::State};
use iced::widget::{button, column, horizontal_space, row, text, text_input};

use iced::{Color, Element, Sandbox};
use ron::ser::PrettyConfig;
pub const NEW_EP: Color = Color::from_rgb(0.2, 0.6, 0.5);
pub const STANDARD: Color = Color::from_rgb(0.2, 0.2, 0.2);

impl Sandbox for State {
    type Message = Message;
    fn new() -> Self {
        let base_str = match std::env::var("HOME") {
            Ok(p) => p,
            Err(_e) => ".".into(),
        };
        let path: PathBuf = ["/"]
            .into_iter()
            .chain(base_str.split("/"))
            .chain([".local", "share", "webworm", "bookmarks.ron"])
            .collect();
        let content = std::fs::read_to_string(path.clone());
        let mut state = match content {
            Ok(content) => ron::de::from_str(content.as_str()).unwrap(),
            _ => {
                println!("No bookmarks found");
                State::default()
            }
        };
        state.new_release = state.entries.iter().map(|entry| entry.has_new()).collect();
        state.config_file_path = path;
        state
    }
    fn title(&self) -> String {
        "webworm".into()
    }
    fn view(&self) -> Element<Message> {
        let header = column![row![text("Webworm").size(38)]];

        let add = row![
            button("Add").on_press(Message::AddEntryBuilder),
            text_input(
                "Title",
                self.entry_builder.name.as_str(),
                Message::NameChanged,
            ),
            text_input("Url", self.entry_builder.site.as_str(), Message::UrlChanged,)
        ];
        let main_column = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let color = if self.new_release[index] {
                    NEW_EP
                } else {
                    STANDARD
                };
                row![
                    button("next").on_press(Message::IncrementEntry(index)),
                    button("copy").on_press(Message::CopyLink(index)),
                    button("fetch").on_press(Message::Fetch(index)),
                    button("x").on_press(Message::RemoveEntry(index)),
                    text(entry.name.as_str()).size(26).style(color),
                    horizontal_space(20),
                    text(&entry.site.as_str()[12..34]).size(26).style(color),
                ]
                .spacing(10)
                .align_items(iced::Alignment::Fill)
            })
            .fold(header, |column, entry| column.push(entry));
        let main_column = main_column.push(add);

        main_column.spacing(20).into()
    }
    fn update(&mut self, message: Message) {
        let needs_saving = message.needs_saving();
        match message {
            Message::IncrementEntry(index) => {
                self.entries[index].next();
            }
            Message::AddEntryBuilder => {
                self.entries.push(self.entry_builder.clone().build());
                self.entry_builder = EntryBuilder::default();
                self.new_release
                    .push(self.entries.last().unwrap().has_new());
            }
            Message::NameChanged(input) => {
                self.entry_builder.name = input;
            }
            Message::UrlChanged(input) => {
                self.entry_builder.site = input;
            }
            Message::RemoveEntry(index) => {
                self.entries.remove(index);
            }
            Message::Fetch(index) => {
                self.new_release[index] = self.entries[index].has_new();
            }
            Message::CopyLink(index) => {
                let _e = cli_clipboard::set_contents(self.entries[index].site.clone());
            }
        }
        if needs_saving {
            let content = ron::ser::to_string_pretty(&self, PrettyConfig::default()).unwrap();

            let mut file = File::create(self.config_file_path.clone()).unwrap();
            let _e = file.write_all(content.as_bytes());
        }
    }
}
