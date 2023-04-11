use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use webpage::{Webpage, WebpageOptions};

use crate::identifier::Identifier;

/// App state
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct State {
    pub entries: Vec<Entry>,
    #[serde(skip)]
    pub new_release: Vec<bool>,
    #[serde(skip)]
    pub entry_builder: EntryBuilder,
    #[serde(skip)]
    pub config_file_path: PathBuf,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EntryBuilder {
    pub name: String,
    pub site: String,
}
impl EntryBuilder {
    pub fn build(self) -> Entry {
        Entry {
            name: self.name,
            site: self.site,
            next: NextSite {
                iden: Identifier::NumberOcc(0),
            },
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    pub name: String,
    pub site: String,
    pub next: NextSite,
}
impl Entry {
    pub fn next(&mut self) {
        self.site = self.next.next(self.site.as_str());
    }
    pub fn has_new(&self) -> bool {
        let url = self.site.as_str();
        let mut options = WebpageOptions::default();
        options.timeout = Duration::from_secs(3);
        let Ok(info) = Webpage::from_url(url, options) else {
            return false;
        };

        // the HTTP transfer
        //
        // the parsed HTML info
        let Some(title) = info.html.title else {
                return false;
            };
        !title.contains("404")
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct NextSite {
    pub iden: Identifier,
}
impl NextSite {
    fn next(&self, current: &str) -> String {
        let mut next = current.to_string();
        let start = self.iden.start(current);
        let end = self.iden.end(current);

        let episode = &current[start..end];
        // replace the current with the next episode
        next.replace_range(
            start..end,
            (episode.parse::<u32>().unwrap() + 1).to_string().as_str(),
        );

        next
    }
}
