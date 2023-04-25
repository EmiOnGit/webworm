use clap::{arg, command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Prints additional debug information
    #[arg(short, long)]
    pub debug: bool,
    /// The path to the bookmarks
    /// If non is given the default ($HOME/.local/share/webworm/bookmarks.ron) is used
    #[arg(short, long)]
    pub bookmark_file: Option<String>,
    /// Advances the bookmarks if possible
    /// only advances the bookmarks selected
    #[arg(short, long)]
    pub advance: bool,
    /// The names of the bookmarks
    /// Can contain globs which will be matched against the names
    /// If no names are given, all will be used
    pub names: Vec<String>,
    /// returns the links instead of the names
    #[arg(short, long)]
    pub link: bool,
}
