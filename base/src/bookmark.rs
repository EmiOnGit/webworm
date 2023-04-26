use std::time::Duration;

use log::warn;
use serde::Deserialize;
use serde::Serialize;
use webpage::Webpage;
use webpage::WebpageOptions;

#[derive(Debug, Deserialize, Serialize)]
pub struct Bookmarks(pub Vec<Bookmark>);
impl Bookmarks {
    /// parses the string to a [`Bookmark`] and pushes the result to the bookmarks
    pub fn push(&mut self, bookmark: &str) -> Result<(), BookmarkInsertError> {
        let Ok(bookmark) = ron::from_str::<Bookmark>(bookmark) else {
            return Err(BookmarkInsertError::WrongFormat);
        };

        if self.0.iter().any(|b| b.name == bookmark.name) {
            return Err(BookmarkInsertError::NameAlreadyFound);
        }
        self.0.push(bookmark);
        Ok(())
    }
    pub fn push_entry(&mut self, bookmark: Bookmark) -> Result<(), BookmarkInsertError> {
        if self.0.iter().any(|b| b.name == bookmark.name) {
            return Err(BookmarkInsertError::NameAlreadyFound);
        }
        self.0.push(bookmark);
        Ok(())
    }
    pub fn advance(&mut self, name: &str) -> Option<&mut Bookmark> {
        if let Some(bookmark) = self.0.iter_mut().find(|bookmark| bookmark.name == name) {
            bookmark.advance();
            return Some(bookmark);
        }
        return None;
    }
    pub fn get(&self, name: &str) -> Option<&Bookmark> {
        self.0.iter().find(|bookmark| bookmark.name == name)
    }
    pub fn remove(&mut self, name: &str) {
        let index = self.0.iter().position(|bookmark| bookmark.name == name);
        if let Some(index) = index {
            self.0.remove(index);
        }
    }
    pub fn previous(&mut self, name: &str) -> Option<&mut Bookmark> {
        if let Some(bookmark) = self.0.iter_mut().find(|bookmark| bookmark.name == name) {
            bookmark.previous();
            return Some(bookmark);
        }
        None
    }
    pub fn update(&mut self, name: &str) -> bool {
        for bookmark in self.0.iter_mut().filter(|bookmark| bookmark.name == name) {
            return bookmark.can_advance();
        }
        return false;
    }
}
#[derive(Debug)]
pub enum BookmarkInsertError {
    NameAlreadyFound,
    WrongFormat,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bookmark {
    /// The name of the bookmark. This can be the title of the series.
    /// The name is assumed to be unique and can be used as uniquekey
    pub name: String,
    /// The url of the bookmark. Note that the episode number is surrounded by "{{ }}"
    pub url: Url,
    /// Since pinging the website for the next episode can be taxing the result is saved.
    #[serde(default)]
    pub can_advance: bool,
}
impl Bookmark {
    /// Returns if the bookmark can advance to the next episode.
    /// Internally just increments the episode by one and tries to ping it
    ///
    /// The result is cached for each bookmark
    pub fn can_advance(&mut self) -> bool {
        if self.can_advance {
            return true;
        }
        let options = WebpageOptions {
            // Shouldn't take to long to ping the website
            timeout: Duration::from_secs(1),
            ..Default::default()
        };
        let next_url = self.url.next().get();
        let Ok(info) = Webpage::from_url(next_url.as_str(), options) else {
            warn!("Couldn't reach url for entry with name {}", self.name);
            return false;
        };

        let Some(title) = info.html.title else {
                return false;
            };
        // TODO Should properly test the return
        self.can_advance = !title.contains("404");
        self.can_advance
    }
    pub fn advance(&mut self) {
        self.can_advance = false;
        self.url = self.url.next();
    }
    pub fn previous(&mut self) {
        self.can_advance = false;
        self.url = self.url.previous();
    }
    pub fn matches(&self, patterns: &[glob::Pattern]) -> bool {
        if patterns.is_empty() {
            return true;
        };
        patterns
            .iter()
            .any(|pattern| pattern.matches(self.name.as_str()))
    }
}

/// A bookmark url is the internal representation of the url for the bookmarks.
/// it uses the format https://example_url{{episode number}}/somethingsomething
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct Url(pub String);
impl Url {
    /// Returns the properly formated url for the browser.
    ///
    /// This removes the "{{}}" symbols which indicate the current episode
    pub fn get(&self) -> String {
        self.0.clone().replace("{{", "").replace("}}", "")
    }
    /// Returns the start and end index of the episode marker.
    /// The current episode is assumed to be in following format "{{episode}}"
    fn episode_position(&self) -> (usize, usize) {
        let start = self.0.find("{{").unwrap() + 2;
        let end = self.0.find("}}").unwrap();
        (start, end)
    }
    /// Returns the current episode
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
    pub fn previous(&self) -> Url {
        let (start, end) = self.episode_position();
        let current_episode = self.0[start..end].parse::<usize>().unwrap();
        let previous_episode = current_episode - 1;
        let mut previous = self.0.clone();
        previous.replace_range(start..end, previous_episode.to_string().as_str());
        Url(previous)
    }
}
