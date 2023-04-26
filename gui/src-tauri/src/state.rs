use std::sync::Mutex;

use base::bookmark::Bookmarks;

pub struct Storage {
    pub bookmarks: Mutex<Bookmarks>,
}
impl Default for Storage {
    fn default() -> Self {
        let path = base::io::bookmark_path();
        let bookmarks = base::io::load_bookmarks(&path);
        Storage {
            bookmarks: Mutex::new(bookmarks),
        }
    }
}

impl Storage {
    pub fn save(&self) {
        let path = base::io::bookmark_path();
        let bookmarks = self.bookmarks.lock().unwrap();

        let _ = base::io::save_bookmarks(&bookmarks, &path);
    }
}
