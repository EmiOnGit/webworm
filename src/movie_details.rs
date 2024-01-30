use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub id: usize,
    seasons: Vec<Season>,
    in_production: bool,
    last_air_date: String,
    pub number_of_seasons: usize,
    pub number_of_episodes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    id: usize,
    episode_count: usize,
    season_number: usize,
    overview: String,
    poster_path: String,
}
