#![windows_subsystem = "windows"]

pub mod bookmark;
pub mod filter;
pub mod gui;
pub mod icons;
pub mod id;
pub mod link;
pub mod message;
pub mod movie;
pub mod movie_details;
pub mod response;
pub mod save;
pub mod state;
pub mod tmdb;
pub mod update;
pub mod view;
use anyhow::Result;
use iced::{window, Application, Settings, Size};

use crate::gui::App;
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{filter::FilterFn, layer::SubscriberExt, util::SubscriberInitExt};

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
        debug = cmd.as_str() == "debug";
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
            size: Size::new(800.0, 500.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    })?;
    Ok(())
}
