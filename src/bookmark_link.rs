use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkLink {
    link: Vec<LinkPart>,
}
impl BookmarkLink {
    /// Creates a new Bookmark link.
    /// Panics if the link is not valid
    pub fn new(link: Vec<LinkPart>) -> Result<BookmarkLink, LinkError> {
        BookmarkLink::is_valid_link(&link)?;
        Ok(BookmarkLink { link })
    }
    /// Converts the internal link representation to the finished url
    pub fn url(&self) -> String {
        self.link
            .clone()
            .into_iter()
            .map(|part| match part {
                LinkPart::Const(s) => s,
                LinkPart::Episode(i) => i.to_string(),
                LinkPart::Season(i) => i.to_string(),
            })
            .collect()
    }

    /// A link is valid if it contains:
    /// * at least one const part
    /// * exactly one episode
    /// * a maximum of one season
    fn is_valid_link(link: &[LinkPart]) -> Result<(), LinkError> {
        let mut consts = 0;
        let mut episode = 0;
        let mut season = 0;
        for part in link {
            match part {
                LinkPart::Const(_) => consts += 1,
                LinkPart::Episode(_) => episode += 1,
                LinkPart::Season(_) => season += 1,
            }
        }
        if consts == 0 {
            warn!("a bookmark link should always include a const part");
            Err(LinkError::NoConstPart)
        } else if episode == 0 {
            warn!("a bookmark link should always include one episode");
            Err(LinkError::NoEpisode)
        } else if episode > 1 {
            warn!("a bookmark link should always include one episode");
            Err(LinkError::ToManyEpisodes)
        } else if season > 1 {
            warn!("a bookmark link shouldn't include more than one season");
            Err(LinkError::ToManySeasons)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkPart {
    Const(String),
    Episode(u16),
    Season(u16),
}
#[derive(Clone, Debug, PartialEq)]
pub enum LinkError {
    NoConstPart,
    NoEpisode,
    ToManySeasons,
    ToManyEpisodes,
}
