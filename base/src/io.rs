use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use ron::ser::PrettyConfig;

use crate::bookmark::Bookmarks;

pub fn bookmark_path() -> PathBuf {
    if cfg!(windows) {
        return PathBuf::from("./bookmarks.ron");
    }
    let base_str = match std::env::var("HOME") {
        Ok(p) => p,
        Err(_e) => ".".into(),
    };
    ["/"]
        .into_iter()
        .chain(base_str.split("/"))
        .chain([".local", "share", "webworm", "bookmarks.ron"])
        .collect()
}
pub fn load_bookmarks(path: &PathBuf) -> Bookmarks {
    if path.exists() {
        let content = fs::read_to_string(path.clone()).unwrap();
        return ron::from_str(content.as_str()).unwrap();
    }
    let _ = File::create(&path);
    Bookmarks(Vec::new())
}
pub fn save_bookmarks(bookmarks: &Bookmarks, path: &PathBuf) {
    let content = ron::ser::to_string_pretty(bookmarks, PrettyConfig::default()).unwrap();
    let mut file = File::create(path).unwrap();
    let _e = file.write_all(content.as_bytes());
}
