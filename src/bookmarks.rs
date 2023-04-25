use std::time::Duration;

use clap::Parser;
use log::info;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use webpage::Webpage;
use webpage::WebpageOptions;

use crate::loading::load_path;
use crate::loading::save;
use crate::{cli::Args, loading::load};
#[derive(Debug, Deserialize, Serialize)]
pub struct Bookmarks(pub Vec<Bookmark>);
impl Bookmarks {
    pub fn filter(&mut self, args: &Args) {
        if args.names.is_empty() {
            return;
        }
        let patterns: Vec<_> = args
            .names
            .iter()
            .map(|name| glob::Pattern::new(name).unwrap())
            .collect();
        self.0 = self
            .0
            .clone()
            .into_iter()
            .filter(|bookmark| {
                patterns
                    .iter()
                    .any(|name| name.matches(bookmark.name.as_str()))
            })
            .collect();
        println!("matches of the filter {}", self.0.len());
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bookmark {
    pub name: String,
    pub url: Url,
    #[serde(default)]
    can_advance: bool,
}
/// A bookmark url is the internal representation of the url for the bookmarks.
/// it uses the format https://example_url{{episode number}}/somethingsomething
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct Url(pub String);
impl Url {
    /// gets the properly formated string
    pub fn get(&self) -> String {
        self.0.clone().replace("{{", "").replace("}}", "")
    }
    fn episode_position(&self) -> (usize, usize) {
        let start = self.0.find("{{").unwrap() + 2;
        let end = self.0.find("}}").unwrap();
        (start, end)
    }
    pub fn episode(&self) -> usize {
        let (start, end) = self.episode_position();
        self.0[start..end].parse::<usize>().unwrap()
    }
    /// Returns the next url of the bookmark
    pub fn next(&self) -> Url {
        let (start, end) = self.episode_position();
        let current_episode = self.0[start..end].parse::<usize>().unwrap();
        let next_episode = current_episode + 1;
        let mut next = self.0.clone();
        next.replace_range(start..end, next_episode.to_string().as_str());
        Url(next)
    }
}
impl Bookmark {
    pub fn can_advance(&mut self) -> bool {
        if self.can_advance {
            return true;
        }
        let mut options = WebpageOptions::default();
        options.timeout = Duration::from_secs(3);
        let next_url = self.url.next().get();
        let Ok(info) = Webpage::from_url(next_url.as_str(), options) else {
            warn!("Couldn't reach url for entry with name {}", self.name);
            return false;
        };

        let Some(title) = info.html.title else {
                return false;
            };
        let can_advance = !title.contains("404");
        self.can_advance = can_advance;
        self.can_advance
    }
}
