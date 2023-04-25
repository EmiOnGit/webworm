use clap::Parser;
use log::info;

use crate::loading::load_path;
use crate::loading::save;
use crate::{cli::Args, loading::load};
use simplelog::Config;
use simplelog::LevelFilter;
mod bookmarks;
mod cli;
mod loading;

fn main() {
    let args = Args::parse();
    if args.debug {
        info!("setup logging");
        let _ = simplelog::SimpleLogger::init(LevelFilter::Info, Config::default());
    }
    let path = load_path(&args);
    let bookmarks = load(path.clone());
    info!("loaded bookmarks with {:?}", bookmarks);
    let Ok(mut bookmarks) = bookmarks else {
        return;
    };
    bookmarks.filter(&args);
    let v: Vec<String> = bookmarks
        .0
        .iter_mut()
        .filter_map(|bookmark| {
            if !bookmark.can_advance() {
                None
            } else {
                if args.link {
                    Some(bookmark.url.next().get())
                } else {
                    Some(bookmark.name.clone())
                }
            }
        })
        .collect();
    let json_str = serde_json::to_string(&v).unwrap();
    let _ = save(&bookmarks, path);
    println!("{json_str}");
}
