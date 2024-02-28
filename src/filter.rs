use serde::{Deserialize, Serialize};

use crate::id::MovieId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Filter {
    #[default]
    Bookmarks,
    Search,
    Completed,
    Details(MovieId),
}

impl Filter {
    pub fn empty_message(&self) -> &str {
        match self {
            Filter::Bookmarks => "You have no bookmarks yet",
            Filter::Search => "Type in a search  term",
            Filter::Completed => "You have no completed movies yet",
            Filter::Details(_) => "Selected movie has not details",
        }
    }
}
