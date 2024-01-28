use iced::{window, Application, Settings, Size};
use webworm::gui::App;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    App::run(Settings {
        window: window::Settings {
            size: Size::new(500.0, 800.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
