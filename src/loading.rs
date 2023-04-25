use std::path::{Path, PathBuf};

use crate::{bookmarks::Bookmarks, cli::Args};
use anyhow::Result;

/// loads the bookmarks from the file
pub fn load(path: PathBuf) -> Result<Bookmarks> {
    let content = std::fs::read_to_string(path.clone())?;
    let bookmarks = ron::de::from_str(content.as_str())?;
    log::info!("extracted path: {:?}", path);

    Ok(bookmarks)
}
pub fn save(bookmarks: &Bookmarks, path: PathBuf) -> Result<()> {
    let bookmarks = ron::to_string(bookmarks)?;
    let _error = std::fs::write(path, bookmarks)?;
    Ok(())
}
pub fn load_path(args: &Args) -> PathBuf {
    let path: PathBuf = if let Some(ref path) = args.bookmark_file {
        Path::new(path.as_str()).into()
    } else {
        let base_str = match std::env::var("HOME") {
            Ok(p) => p,
            Err(_e) => ".".into(),
        };
        ["/"]
            .into_iter()
            .chain(base_str.split("/"))
            .chain([".local", "share", "webworm", "bookmarks.ron"])
            .collect()
    };
    path
}
