use base::bookmark::Bookmarks;
use base::io::default_bookmark_file;
use base::io::load_bookmarks;
use base::io::save_bookmarks;
use clap::Parser;
use glob::Pattern;
use log::error;
use log::info;
use simplelog::SimpleLogger;

use crate::cli::Args;
use crate::cli::Command;
use crate::cli::ReturnFormatting;
use simplelog::Config;
use simplelog::LevelFilter;
mod cli;

fn main() {
    let args = Args::parse();
    // setup logging.
    // If the simple logger hasn't been initialized, it will discard the logs
    if args.debug {
        let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
        info!("setup logging");
    }
    let path = default_bookmark_file();
    let bookmarks = load_bookmarks(path.clone());

    let Ok(mut bookmarks) = bookmarks else {
        error!("couldn't load bookmarks with error {:?}", bookmarks.unwrap_err());
        return;
    };
    info!("loaded bookmarks with {:?}", bookmarks);

    match args.command {
        Command::Push { entries } => {
            for entry in entries.iter() {
                push_entry(&mut bookmarks, entry.as_str());
            }
        }
        Command::Print { all, glob, format } => {
            let patterns = patterns(glob);
            let format = ReturnFormatting::from(format);
            let result: Vec<String> = bookmarks
                .0
                .iter_mut()
                .filter(|bookmark| bookmark.matches(patterns.as_slice()))
                .filter_map(|bookmark| {
                    if all {
                        return Some(bookmark);
                    }
                    if !bookmark.can_advance() {
                        None
                    } else {
                        Some(bookmark)
                    }
                })
                .map(|bookmark| format.print(bookmark))
                .collect();
            let joined = result.join(" ; ");
            println!("{joined}");
        }
        Command::Advance { glob } => {
            let patterns = patterns(glob);
            bookmarks
                .0
                .iter_mut()
                .filter(|bookmark| bookmark.matches(patterns.as_slice()))
                .filter_map(|bookmark| {
                    if !bookmark.can_advance() {
                        None
                    } else {
                        Some(bookmark)
                    }
                })
                .for_each(|bookmark| {
                    bookmark.advance();
                });
        }
    }
    let _ = save_bookmarks(&bookmarks, path);
}
fn patterns(globs: Vec<String>) -> Vec<Pattern> {
    globs
        .iter()
        .map(|name| glob::Pattern::new(name).unwrap())
        .collect()
}
fn push_entry(bookmarks: &mut Bookmarks, bookmark: &str) {
    let err = bookmarks.push(bookmark);
    if let Err(e) = err {
        error!("couldn't insert bookmark with following error {:?}", e);
    } else {
        info!("successfully added new bookmark");
    }
}
