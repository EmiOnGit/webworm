use base::bookmark::{Bookmark, Url};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsBookmark {
    pub name: String,
    episode: usize,
    url: String,
    has_new: bool,
}
impl JsBookmark {
    pub fn from_bookmark(bookmark: &Bookmark) -> Self {
        JsBookmark {
            name: bookmark.name.clone(),
            episode: bookmark.url.episode(),
            url: bookmark.url.get(),
            has_new: bookmark.can_advance,
        }
    }
    pub fn into_bookmark(&self) -> Result<Bookmark, String> {
        let ep_str = self.episode.to_string();
        let ep_str = ep_str.as_str();
        let open = "{{";
        let end = "}}";
        let url = self
            .url
            .replace(ep_str, format!("{open}{ep_str}{end}").as_str());
        if url == self.url {
            return Err("episode was not found in url".into());
        }
        Ok(Bookmark {
            name: self.name.clone(),
            url: Url(url),
            can_advance: false,
        })
    }
}
