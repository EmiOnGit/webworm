pub mod bookmark;
pub mod bookmark_link;
pub mod filter;
pub mod gui;
pub mod message;
pub mod movie;
pub mod movie_details;
pub mod save;
pub mod state;
pub mod tmdb;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
