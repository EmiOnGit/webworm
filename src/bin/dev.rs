use anyhow::Result;
use iced::{window, Application, Settings, Size};

use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{filter::FilterFn, layer::SubscriberExt, util::SubscriberInitExt};
use webworm::gui::App;

const LOG_IGNORE: [&str; 13] = [
    "vulkan::instance",
    "iced_wgpu::window::compositor",
    "wgpu_core::instance",
    "wgpu_core::resource",
    "wgpu_core::device",
    "wgpu_core::present",
    "wgpu_hal",
    "iced_wgpu::image",
    "iced_wgpu::backend",
    "cosmic_text::font",
    "cosmic_text::buffer",
    "naga",
    "sctk",
];
fn main() -> Result<()> {
    let mut debug = false;
    let mut args = std::env::args();
    args.next();
    if let Some(cmd) = args.next() {
        match cmd.as_str() {
            "debug" => {
                debug = true;
            }
            _ => {}
        }
    }
    let filter = FilterFn::new(move |meta| {
        let target = meta.target();
        let in_ignore_list = LOG_IGNORE.iter().any(|name| target.contains(name));
        if !debug && target.contains("webworm") && target.contains("debug") {
            return false;
        }
        if in_ignore_list {
            meta.level() < &Level::WARN
        } else {
            true
        }
    })
    .with_max_level_hint(if debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    });

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true).pretty();
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
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
