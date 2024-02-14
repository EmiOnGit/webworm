use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub id: usize,
    seasons: Vec<Season>,
    in_production: bool,
    last_air_date: Option<String>,
    number_of_seasons: usize,
    number_of_episodes: usize,
    last_episode_to_air: Option<Episode>,
    next_episode_to_air: Option<Episode>,
}
impl MovieDetails {
    /// Tries to fetch the last published episode.
    /// In case the last episode was not given, the `id` from the movie is used
    pub fn last_published(&self) -> Episode {
        if let Some(last) = &self.last_episode_to_air {
            last.clone()
        } else if let Some(next) = &self.next_episode_to_air {
            let mut last = next.clone();
            if last.episode_number > 1 {
                last.episode_number -= 1;
                last
            } else if last.season_number == 1 {
                last
            } else {
                last.season_number -= 1;
                last.episode_number = self.seasons[last.season_number].episode_count;
                last
            }
        } else {
            Episode {
                id: self.id,
                air_date: self.last_air_date.clone().unwrap_or_default(),
                episode_number: self.number_of_episodes,
                name: "".to_owned(),
                season_number: self.number_of_seasons,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    id: usize,
    episode_count: usize,
    season_number: usize,
    overview: String,
    poster_path: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    id: usize,
    air_date: String,
    pub episode_number: usize,
    name: String,
    pub season_number: usize,
}
