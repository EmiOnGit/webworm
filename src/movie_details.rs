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
    last_episode_to_air: Option<EpisodeDetails>,
    next_episode_to_air: Option<EpisodeDetails>,
    #[serde(skip)]
    fixed: bool,
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
                .find(|season| season.season_number == last_episode_to_air.episode.season_number)
            else {
                panic!("season for last_episode not found");
            };

            if current_season.episode_count < last_episode_to_air.episode.episode_number {
                info!(
                    "movie {} seems to have invalid episode formats. The season should only have {} episodes, but the last episode is {} ",
                    self.id,
                    current_season.episode_count,
                    last_episode_to_air.episode.episode_number
                );
                let episodes_before_season: usize = self
                    .seasons
                    .iter()
                    .filter(|s| s.season_number < last_episode_to_air.episode.season_number)
                    .filter(|s| !s.name.contains("Extras") && !s.name.contains("Specials"))
                    .map(|s| s.episode_count)
                    .sum();
                let Some(episode_number) = last_episode_to_air
                    .episode
                    .episode_number
                    .checked_sub(episodes_before_season)
                else {
                    error!("last_episode_to_air: {last_episode_to_air:?}, episodes_before_season: {episodes_before_season}, seasons: {:?}", self.seasons);
                    panic!()
                };
                self.fixed = true;
                last_episode_to_air.episode.episode_number = episode_number;
            }
        }
        if let Some(next_episode_to_air) = &mut self.next_episode_to_air {
            let Some(current_season) = self
                .seasons
                .iter()
                .find(|season| season.season_number == next_episode_to_air.episode.season_number)
            else {
                panic!("season for last_episode not found");
            };
            if current_season.episode_count < next_episode_to_air.episode.episode_number {
                info!(
                    "movie {} seems to have invalid episode formats. The season should only have {} episodes, but the next episode is {} ",
                    self.id,
                    current_season.episode_count,
                    next_episode_to_air.episode.episode_number
                );
                let episodes_before_season: usize = self
                    .seasons
                    .iter()
                    .filter(|s| s.season_number < next_episode_to_air.episode.season_number)
                    .filter(|s| !s.name.contains("Extras") && !s.name.contains("Specials"))
                    .map(|s| s.episode_count)
                    .sum();
                self.fixed = true;
                next_episode_to_air.episode.episode_number -= episodes_before_season;
            }
        }
    }
    pub fn reformat_for_request(&self, mut episode: Episode) -> Episode {
        if !self.fixed {
            return episode;
        }
        let episodes_before_season: usize = self
            .seasons
            .iter()
            .filter(|s| s.season_number < episode.season())
            .filter(|s| !s.name.contains("Extras") && !s.name.contains("Specials"))
            .map(|s| s.episode_count)
            .sum();
        episode.set_episode(episode.episode() + episodes_before_season);
        episode
    }

    pub fn as_total_episodes<E: Into<Episode> + Clone>(&self, episode: &E) -> TotalEpisode {
        let episode: Episode = episode.clone().into();
        match episode {
            Episode::Seasonal(episode) => {
                let sum_before: usize = self
                    .seasons
                    .iter()
                    .filter(|season| {
                        season.season_number < episode.season_number
                            && !season.name.contains("Specials")
                    })
                    .map(|s| s.episode_count)
                    .sum();
                TotalEpisode {
                    episode: sum_before + episode.episode_number,
                }
            }
            Episode::Total(episode) => episode,
        }
    }
    pub fn as_seasonal_episode<E: Into<Episode> + Clone>(&self, episode: &E) -> SeasonEpisode {
        let episode: Episode = episode.clone().into();
        match episode {
            Episode::Seasonal(episode) => episode,
            Episode::Total(TotalEpisode { mut episode }) => {
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
                        .unwrap_or_else(|| {
                            panic!(
                                "can not find season {} for movie {}",
                                season_number - 1,
                                self.id
                            )
                        });
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
    pub(crate) fn seasons(&self) -> &[Season] {
        &self.seasons
    }
    pub fn next_episode(&self, mut episode: Episode) -> Episode {
        let Some(last_published) = self.last_published() else {
            return episode;
        };
        match &mut episode {
            Episode::Seasonal(ep) => {
                if *ep != last_published.episode {
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
                let TotalEpisode {
                    episode: total_episodes,
                } = self.as_total_episodes(&last_published.episode);
                episode = Episode::Total(TotalEpisode {
                    episode: (ep.episode + 1).min(total_episodes),
                });
            }
        }
        episode
    }
    /// Tries to fetch the last published episode.
    /// In case the last episode was not given, the `id` from the movie is used
    pub fn last_published(&self) -> Option<EpisodeDetails> {
        self.last_episode_to_air.clone()
    }
    pub fn next_episode_to_air(&self) -> Option<EpisodeDetails> {
        self.next_episode_to_air.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    id: usize,
    name: String,
    pub episode_count: usize,
    season_number: usize,
    overview: String,
    poster_path: Option<String>,
}
impl Season {
    /// Returns the season number.
    pub(crate) fn number(&self) -> usize {
        self.season_number
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EpisodeDetails {
    #[serde(flatten)]
    pub episode: SeasonEpisode,
    pub name: String,
    pub air_date: Option<String>,
    pub overview: String,
}
#[derive(Eq, Debug, PartialEq, Serialize, Hash, Deserialize, Clone)]
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
    pub(crate) fn episode(&self) -> usize {
        match self {
            Episode::Seasonal(e) => e.episode_number,
            Episode::Total(e) => e.episode,
        }
    }
    pub(crate) fn season(&self) -> usize {
        match self {
            Episode::Seasonal(e) => e.season_number,
            Episode::Total(_e) => 1,
        }
    }
    pub fn as_info_str(&self) -> String {
        match self {
            Episode::Seasonal(e) => format!("{}E · {}S", e.episode_number, e.season_number),
            Episode::Total(e) => format!("{}E", e.episode),
        }
    }
    /// Increments the episode by one.
    /// It should be checked if the episode exists beforehand.
    pub(crate) fn next_episode(&mut self) {
        match self {
            Episode::Seasonal(e) => e.episode_number += 1,
            Episode::Total(e) => e.episode += 1,
        }
    }

    /// Decrement the episode by one.
    /// This method saturates at 0 instead of panicing
    pub(crate) fn previous_episode(&mut self) {
        match self {
            Episode::Seasonal(e) => e.episode_number = e.episode_number.saturating_sub(1),
            Episode::Total(e) => e.episode = e.episode.saturating_sub(1),
        }
    }
    pub(crate) fn set_episode(&mut self, episode: usize) {
        match self {
            Episode::Seasonal(e) => e.episode_number = episode,
            Episode::Total(e) => e.episode = episode,
        }
    }
    pub(crate) fn set_season(&mut self, season: usize) {
        match self {
            Episode::Seasonal(e) => e.season_number = season,
            Episode::Total(e) => {
                info!("upgrade total episode to seasonal since season was set");
                *self = Episode::Seasonal(SeasonEpisode {
                    episode_number: e.episode,
                    season_number: 2,
                });
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Hash, Deserialize, PartialEq, Eq)]
pub struct SeasonEpisode {
    /// Episode in the season. The first episode of the season should always be 1
    pub episode_number: usize,
    /// The season of the episode. The first season is assumed to be 1
    pub season_number: usize,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct TotalEpisode {
    pub episode: usize,
}

impl TotalEpisode {
    pub fn as_info_str(&self) -> String {
        format!("{}E", self.episode)
    }
}
impl SeasonEpisode {
    pub fn as_info_str(&self) -> String {
        format!("{}E · {}S", self.episode_number, self.season_number)
    }
}
