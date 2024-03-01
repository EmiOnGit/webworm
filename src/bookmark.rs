use std::fmt::Debug;

use iced::widget::image;
use iced::Command;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::id::EpisodeId;
use crate::message::{BookmarkMessage, Message};
use crate::movie::TmdbMovie;
use crate::movie_details::{Episode, TotalEpisode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub movie: TmdbMovie,
    pub current_episode: Episode,
    /// True if the current episode is already watched
    pub finished: bool,
    sync_mode: SyncMode,
}
/// Defines how the bookmark progress should be handled
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SyncMode {
    /// The progress is not synced with the Tmdb database, allowing the bookmark to have a episode that is not registered in the database.
    /// Also the season will never change unless explicitly incremented
    NoSync,
    /// The progress is synced with the Tmdb database.
    /// Therefore the bookmark can only have a `current_episode`, that is also found on tmdb
    /// The season will automatically changed as the last episode of a season is watched
    Tmdb,
}
#[derive(Clone, Debug)]
pub enum Poster {
    Image(image::Handle),
}
impl From<&TmdbMovie> for Bookmark {
    fn from(movie: &TmdbMovie) -> Self {
        Self {
            movie: movie.clone(),
            current_episode: Episode::Total(TotalEpisode { episode: 1 }),
            finished: false,
            sync_mode: SyncMode::Tmdb,
        }
    }
}
impl Bookmark {
    pub fn apply(&mut self, action: BookmarkMessage) -> Command<Message> {
        match action {
            BookmarkMessage::IncrE(details) => match self.sync_mode {
                SyncMode::NoSync => self.current_episode.next_episode(),
                SyncMode::Tmdb => {
                    if let Some(details) = details {
                        debug!("increment bookmark episode {:?}", self);
                        let next_episode = details.next_episode(self.current_episode.clone());
                        self.finished = next_episode == self.current_episode;
                        self.current_episode = next_episode;
                    } else {
                        warn!(
                            "Can not increment episode as the movie details are not loaded id: {}",
                            self.movie.id
                        );
                    }
                }
            },
            BookmarkMessage::DecrE(details) => {
                match self.sync_mode {
                    SyncMode::NoSync => self.current_episode.previous_episode(),
                    SyncMode::Tmdb => {
                        if let Some(details) = details {
                            debug!("decrement bookmark episode {:?}", self);
                            if self.finished {
                                debug!("Since the bookmark was on finished state, finish flag was removed");
                                self.finished = false;
                            } else {
                                let previous_episode =
                                    details.previous_episode(self.current_episode.clone());
                                self.current_episode = previous_episode;
                                self.finished = false;
                            }
                        } else {
                            warn!(
                        "Can not decrement episode as the movie details are not loaded id: {}",
                        self.movie.id
                    );
                        }
                    }
                }
            }
            BookmarkMessage::SetE(episode, details) => {
                let Ok(episode) = episode.parse::<usize>() else {
                    warn!("Tried to parse {episode} as a episode when only numbers are allowed");
                    return Command::none();
                };
                self.finished = false;
                match self.sync_mode {
                    SyncMode::NoSync => self.current_episode.set_episode(episode),
                    SyncMode::Tmdb => {
                        let Some(details) = details else {
                            warn!(
                                "Can not set episode as the movie details are not loaded. id: {}",
                                self.movie.id
                            );
                            return Command::none();
                        };
                        let current_episode = details.as_seasonal_episode(&self.current_episode);
                        if let Some(season) = details
                            .seasons()
                            .iter()
                            .find(|season| season.number() == current_episode.season_number)
                        {
                            if let Some(last_published) = details.last_published() {
                                if last_published.episode.season_number
                                    == current_episode.season_number
                                    && last_published.episode.episode_number < episode
                                {
                                    info!("Attempted to set episode higher than the last published episode. Default to last published here");
                                    self.current_episode = last_published.episode.into();
                                } else {
                                    let episode = season.episode_count.min(episode);
                                    debug!("Set episode to {episode}");
                                    self.current_episode.set_episode(episode);
                                }
                            } else {
                                let episode = season.episode_count.min(episode);
                                self.current_episode.set_episode(episode);
                            }
                        } else {
                            warn!("Couldn't set bookmark {} to episode {episode} since no season was found in the database that matches the bookmark state", self.movie.id);
                        }
                    }
                }
            }
            BookmarkMessage::SetS(new_season, details) => {
                let Ok(new_season) = new_season.parse::<usize>() else {
                    warn!("Tried to parse {new_season} as a season when only numbers are allowed");
                    return Command::none();
                };

                self.finished = false;
                match self.sync_mode {
                    SyncMode::NoSync => self.current_episode.set_season(new_season),
                    SyncMode::Tmdb => {
                        let Some(details) = details else {
                            warn!(
                                "Can not set season as the movie details are not loaded. id: {}",
                                self.movie.id
                            );
                            return Command::none();
                        };
                        if let Some(season) = details
                            .seasons()
                            .iter()
                            .find(|season| season.number() == new_season)
                        {
                            self.current_episode.set_season(new_season);
                            if let Some(last_published) = details.last_published() {
                                if last_published.episode.season_number == new_season
                                    && last_published.episode.episode_number
                                        < self.current_episode.episode()
                                {
                                    self.current_episode = last_published.episode.into();
                                } else {
                                    let episode =
                                        season.episode_count.min(self.current_episode.episode());
                                    self.current_episode.set_episode(episode);
                                }
                                // TODO Combine as let chains are stabilized. see https://github.com/rust-lang/rust/issues/53667
                            } else {
                                let episode =
                                    season.episode_count.min(self.current_episode.episode());
                                self.current_episode.set_episode(episode);
                            }
                        } else {
                            warn!("Couldn't set bookmark {} to season {new_season} since no such season was found in the database", self.movie.id);
                        }
                    }
                }
            }
            BookmarkMessage::ToggleSync => {
                match self.sync_mode {
                    SyncMode::NoSync => self.sync_mode = SyncMode::Tmdb,
                    SyncMode::Tmdb => self.sync_mode = SyncMode::NoSync,
                }
                info!("Toggle sync mode of bookmark with id {}", self.movie.id);
            }
        }
        Command::none()
    }
    pub fn current_episode_id(&self) -> EpisodeId {
        EpisodeId(self.movie.id, self.current_episode.clone())
    }
}
