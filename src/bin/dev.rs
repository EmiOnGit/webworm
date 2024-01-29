use anyhow::Result;
use iced::{window, Application, Settings, Size};

use webworm::gui::App;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    App::run(Settings {
        window: window::Settings {
            size: Size::new(500.0, 800.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    })?;
    Ok(())
}
