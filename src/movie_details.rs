use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::id::MovieId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub id: MovieId,
    seasons: Vec<Season>,
    in_production: bool,
    last_air_date: Option<String>,
    number_of_seasons: usize,
    number_of_episodes: usize,
    last_episode_to_air: Option<SeasonEpisode>,
    next_episode_to_air: Option<SeasonEpisode>,
}
impl MovieDetails {
    /// Sometimes shows encode the `SeasonEpisode` to not have resetting episodes counts.
    /// We have to check that and fix if needed as our calculations can not handle both formats.
    pub fn fix_episode_formats(&mut self) {
        if self.seasons.len() < 2 {
            return;
        }
        if let Some(last_episode_to_air) = &mut self.last_episode_to_air {
            let Some(current_season) = self
                .seasons
                .iter()
                .find(|season| season.season_number == last_episode_to_air.season_number)
            else {
                panic!("season for last_episode not found");
            };

            if current_season.episode_count < last_episode_to_air.episode_number {
                info!(
                    "movie {} seems to have invalid episode formats. The season should only have {} episodes, but the last episode is {} ",
                    self.id,
                    current_season.episode_count,
                    last_episode_to_air.episode_number
                );
                let episodes_before_season: usize = self
                    .seasons
                    .iter()
                    .filter(|s| s.season_number < last_episode_to_air.season_number)
                    .filter(|s| !s.name.contains("Extras") && !s.name.contains("Specials"))
                    .map(|s| s.episode_count)
                    .sum();
                let Some(episode_number) = last_episode_to_air
                    .episode_number
                    .checked_sub(episodes_before_season)
                else {
                    error!("last_episode_to_air: {last_episode_to_air:?}, episodes_before_season: {episodes_before_season}, seasons: {:?}", self.seasons);
                    panic!()
                };
                last_episode_to_air.episode_number = episode_number;
            }
        }
        if let Some(next_episode_to_air) = &mut self.next_episode_to_air {
            let Some(current_season) = self
                .seasons
                .iter()
                .find(|season| season.season_number == next_episode_to_air.season_number)
            else {
                panic!("season for last_episode not found");
            };
            if current_season.episode_count < next_episode_to_air.episode_number {
                info!(
                    "movie {} seems to have invalid episode formats. The season should only have {} episodes, but the next episode is {} ",
                    self.id,
                    current_season.episode_count,
                    next_episode_to_air.episode_number
                );
                let episodes_before_season: usize = self
                    .seasons
                    .iter()
                    .filter(|s| s.season_number < next_episode_to_air.season_number)
                    .filter(|s| !s.name.contains("Extras") && !s.name.contains("Specials"))
                    .map(|s| s.episode_count)
                    .sum();
                next_episode_to_air.episode_number -= episodes_before_season;
            }
        }
    }
    pub fn as_total_episodes(&self, episode: &SeasonEpisode) -> TotalEpisode {
        let sum_before: usize = self
            .seasons
            .iter()
            .filter(|season| {
                season.season_number < episode.season_number && !season.name.contains("Specials")
            })
            .map(|s| s.episode_count)
            .sum();
        error!("sum before {}", sum_before);
        error!("seasons: {:?}", self.seasons);
        TotalEpisode {
            episode: sum_before + episode.episode_number,
        }
    }
    pub fn as_seasonal_episode(&self, episode: &TotalEpisode) -> SeasonEpisode {
        let mut episode = episode.episode;
        let mut season_number = 1;
        for season in &self.seasons {
            if season.name.contains("Specials") {
                continue;
            }
            if season.episode_count > episode {
                return SeasonEpisode {
                    episode_number: episode,
                    season_number,
                };
            }
            season_number = season.season_number;
            episode -= season.episode_count;
        }
        SeasonEpisode {
            episode_number: episode,
            season_number,
        }
    }
    pub fn previous_episode(&self, episode: Episode) -> Episode {
        match episode {
            Episode::Seasonal(SeasonEpisode {
                episode_number,
                season_number,
            }) => {
                if episode_number > 1 {
                    Episode::Seasonal(SeasonEpisode {
                        episode_number: episode_number - 1,
                        season_number,
                    })
                } else if season_number > 1 {
                    let previous_season = self
                        .seasons
                        .iter()
                        .find(|s| s.season_number == season_number - 1)
                        .unwrap_or_else(|| panic!("can not find season {} for movie {}",
                            season_number - 1,
                            self.id));
                    Episode::Seasonal(SeasonEpisode {
                        episode_number: previous_season.episode_count,
                        season_number: previous_season.season_number,
                    })
                } else {
                    episode
                }
            }
            Episode::Total(ep) => Episode::Total(TotalEpisode {
                episode: (ep.episode - 1).max(1),
            }),
        }
    }
    pub fn next_episode(&self, mut episode: Episode) -> Episode {
        let last_published = self.last_published();
        match &mut episode {
            Episode::Seasonal(ep) => {
                if *ep != last_published {
                    let current_season = self
                        .seasons
                        .iter()
                        .find(|s| s.season_number == ep.season_number);
                    let Some(current_season) = current_season else {
                        error!("season not found");
                        return episode;
                    };
                    if current_season.episode_count == ep.episode_number {
                        ep.episode_number = 1;
                        ep.season_number += 1;
                    } else {
                        ep.episode_number += 1;
                    }
                }
            }
            Episode::Total(ep) => {
                let last_published = self.last_published();
                let TotalEpisode {
                    episode: total_episodes,
                } = self.as_total_episodes(&last_published);
                episode = Episode::Total(TotalEpisode {
                    episode: (ep.episode + 1).min(total_episodes),
                });
            }
        }
        episode
    }
    // / Tries to fetch the last published episode.
    // / In case the last episode was not given, the `id` from the movie is used
    pub fn last_published(&self) -> SeasonEpisode {
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
            SeasonEpisode {
                episode_number: self.number_of_episodes,
                season_number: self.number_of_seasons,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    id: usize,
    name: String,
    episode_count: usize,
    season_number: usize,
    overview: String,
    poster_path: Option<String>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Episode {
    Seasonal(SeasonEpisode),
    Total(TotalEpisode),
}
impl From<SeasonEpisode> for Episode {
    fn from(value: SeasonEpisode) -> Self {
        Self::Seasonal(value)
    }
}
impl From<TotalEpisode> for Episode {
    fn from(value: TotalEpisode) -> Self {
        Self::Total(value)
    }
}
impl Episode {
    pub fn as_info_str(&self) -> String {
        match self {
            Episode::Seasonal(e) => format!("{}E · {}S", e.episode_number, e.season_number),
            Episode::Total(e) => format!("{}E", e.episode),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonEpisode {
    /// Episode in the season. The first episode of the season should always be 1
    pub episode_number: usize,
    /// The season of the episode. The first season is assumed to be 1
    pub season_number: usize,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TotalEpisode {
    pub episode: usize,
}
