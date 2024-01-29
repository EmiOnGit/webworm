use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;

const EPISODE_PLACEHOLDER: &str = "{e}";
const SEASON_PLACEHOLDER: &str = "{s}";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkLink {
    pub link_parts: Vec<LinkPart>,
    pub string_link: String,
}
fn parse_link(link: &str) -> Vec<LinkPart> {
    let mut parts = Vec::new();
    parse_link_impl(link, &mut parts);
    parts
}
/// Top-down recursive tokenizer
fn parse_link_impl(link: &str, parsed: &mut Vec<LinkPart>) {
    if link.is_empty() {
        return;
    }
    let (part, rest) = match (
        link.find(EPISODE_PLACEHOLDER),
        link.find(SEASON_PLACEHOLDER),
    ) {
        (None, None) => (LinkPart::Const(link.to_owned()), None),
        (_, Some(0)) => (LinkPart::Season, Some(&link[SEASON_PLACEHOLDER.len()..])),
        (Some(0), _) => (LinkPart::Episode, Some(&link[EPISODE_PLACEHOLDER.len()..])),
        (None, Some(i)) => (LinkPart::Const(link[..i].to_owned()), Some(&link[i..])),
        (Some(i), None) => (LinkPart::Const(link[..i].to_owned()), Some(&link[i..])),
        (Some(x), Some(y)) if x > y => (LinkPart::Const(link[..y].to_owned()), Some(&link[y..])),
        (Some(x), Some(_y)) => (LinkPart::Const(link[..x].to_owned()), Some(&link[x..])),
    };
    parsed.push(part);
    if let Some(rest) = rest {
        parse_link_impl(rest, parsed);
    }
}
impl BookmarkLink {
    pub fn url(&self, episode: usize, season: usize) -> String {
        self.link_parts
            .clone()
            .into_iter()
            .map(|part| match part {
                LinkPart::Const(s) => s.clone(),
                LinkPart::Episode => episode.to_string(),
                LinkPart::Season => season.to_string(),
            })
            .collect()
    }
    /// Creates a new Bookmark link.
    /// Panics if the link is not valid
    pub fn new(link: &str) -> Result<BookmarkLink, LinkError> {
        let link_parts = parse_link(link);

        BookmarkLink::is_valid_link(&link_parts)?;
        Ok(BookmarkLink {
            link_parts,
            string_link: link.into(),
        })
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
                LinkPart::Episode => episode += 1,
                LinkPart::Season => season += 1,
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
    Episode,
    Season,
}
#[derive(Clone, Debug, PartialEq)]
pub enum LinkError {
    NoConstPart,
    NoEpisode,
    ToManySeasons,
    ToManyEpisodes,
}
