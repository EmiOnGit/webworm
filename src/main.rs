use iced::{Sandbox, Settings};
use state::State;
mod identifier;
mod messages;
mod state;
mod view;
fn main() -> iced::Result {
    State::run(Settings::default())
}
