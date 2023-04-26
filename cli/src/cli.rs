use base::bookmark::Bookmark;
use clap::{arg, command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Prints additional debug information
    #[arg(short, long)]
    pub debug: bool,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Pushes a new bookmark which will be saved
    /// The string should be in the following format "(name: "example name", url: "https://testurl_episode{{33}}")"
    Push {
        #[arg(short, long)]
        entries: Vec<String>,
    },
    Print {
        /// Bookmarks which doesn't have a new episode wont be filtered
        #[arg(short, long)]
        all: bool,
        /// The names of the bookmarks
        /// Can contain globs which will be matched against the names
        /// If no names are given, all will be used
        #[arg(short, long)]
        glob: Vec<String>,
        #[arg(short, long)]
        format: Option<String>,
    },
    Advance {
        /// The names of the bookmarks
        /// Can contain globs which will be matched against the names
        /// If no names are given, all will be used
        #[arg(short, long)]
        glob: Vec<String>,
    },
}
#[derive(Default, Debug, Clone)]
pub enum ReturnFormatting {
    #[default]
    Name,
    Url,
    Episode,
}
impl ReturnFormatting {
    pub fn print(&self, bookmark: &Bookmark) -> String {
        match self {
            Self::Name => bookmark.name.clone(),
            Self::Url => bookmark.url.get(),
            Self::Episode => format!("{}, {}", bookmark.name, bookmark.url.episode()),
        }
    }
}
impl From<Option<String>> for ReturnFormatting {
    fn from(value: Option<String>) -> Self {
        let Some(value) = value else {
            return Self::default();
        };
        let value = value.to_lowercase();
        if value.contains("url") || value.contains("link") {
            return Self::Url;
        }
        if value.contains("ep") {
            return Self::Episode;
        }
        return Self::Name;
    }
}
