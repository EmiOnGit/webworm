use iced::{widget, window, Command};
use tracing::{debug, error, info, warn};

use crate::{
    bookmark::{Bookmark, Poster},
    filter::Filter,
    id::{EpisodeId, MovieIndex},
    link::Link,
    message::{BookmarkMessage, LinkMessage, Message, ShiftPressed},
    save::load_poster,
    state::{InputKind, State},
    tmdb::{self, RequestType, TmdbConfig},
};
pub struct StateUpdate {
    command: Command<Message>,
    save: bool,
}
impl StateUpdate {
    pub fn new(command: Command<Message>) -> Self {
        StateUpdate {
            command,
            save: true,
        }
    }
    pub fn just_saved(mut self) -> Self {
        self.save = false;
        self
    }
    pub fn has_just_saved(&self) -> bool {
        !self.save
    }
    pub fn command(self) -> Command<Message> {
        self.command
    }
    pub fn add_command(self, command: Command<Message>) -> Self {
        let save = self.save;
        let old = self.command();
        StateUpdate {
            command: Command::batch([old, command]),
            save,
        }
    }
}
impl Default for StateUpdate {
    fn default() -> Self {
        StateUpdate {
            command: Command::none(),
            save: true,
        }
    }
}
impl State {
    pub fn update_state(&mut self, message: Message) -> StateUpdate {
        let mut update = None;
        match message {
            Message::Loaded(_) => warn!(
                "received message {:?} when already in loaded state",
                message
            ),
            Message::Saved(_) => {
                self.saving = false;
                update = StateUpdate::default().just_saved().into();
            }
            Message::InputChanged(kind, input) => self.input_caches[kind] = input,
            Message::InputSubmit(input) => match input {
                InputKind::SearchField => match self.filter {
                    Filter::Search => {
                        let request = RequestType::TvSearch {
                            query: self.input_caches[input].clone(),
                        };
                        update = self.update_state(Message::ExecuteRequest(request)).into();
                        self.input_caches[input] = String::new();
                    }
                    Filter::Details(_) => warn!("Input submit in details view received"),
                    Filter::Bookmarks | Filter::Completed => {
                        info!("ignore input submit in current filter")
                    }
                },
                InputKind::EpisodeInput => {
                    let Filter::Details(movie_id) = self.filter else {
                        return StateUpdate::default();
                    };
                    update = self
                        .update_state(Message::BookmarkMessage(
                            movie_id,
                            BookmarkMessage::SetE(
                                self.input_caches[input].clone(),
                                self.movie_details.get(&movie_id).cloned(),
                            ),
                        ))
                        .into();
                }
                InputKind::SeasonInput => {
                    let Filter::Details(movie_id) = self.filter else {
                        return StateUpdate::default();
                    };
                    update = self
                        .update_state(Message::BookmarkMessage(
                            movie_id,
                            BookmarkMessage::SetS(
                                self.input_caches[input].clone(),
                                self.movie_details.get(&movie_id).cloned(),
                            ),
                        ))
                        .into();
                }
                InputKind::LinkInput => {
                    let Filter::Details(movie_id) = self.filter else {
                        return StateUpdate::default();
                    };
                    let input = &self.input_caches[input];
                    let link = Link::new(input);
                    let Ok(link) = link else {
                        error!("{input} is not a valid link. Error {link:?}");
                        return StateUpdate::default();
                    };
                    self.links.insert(movie_id, link);
                }
            },

            Message::ExecuteRequest(request) => {
                let config = self.config();
                let mut send_request = request.clone();
                let cmd = if let RequestType::Poster { id, path } = request {
                    Command::perform(load_poster(id, path.clone(), config), move |data| {
                        Message::RequestPoster(id, data.ok())
                    })
                } else {
                    if let RequestType::EpisodeDetails { id } = &request {
                        if let Some(details) = self.movie_details.get(&id.0) {
                            let fixed_episode =
                                EpisodeId(id.0, details.reformat_for_request(id.1.clone()));
                            let reformated_request =
                                RequestType::EpisodeDetails { id: fixed_episode };
                            send_request = reformated_request;
                        }
                    };
                    Command::perform(tmdb::send_request(config, send_request), |data| {
                        Message::RequestResponse(data.ok(), request)
                    })
                };
                update = StateUpdate::new(cmd).into();
            }
            Message::RequestResponse(text, query) => {
                let Some(text) = text else {
                    warn!("unsuccessfull request {:?}", query);
                    return StateUpdate::default();
                };
                let cmd = match query {
                    RequestType::TvSearch { .. } => self.response_tv_search(text),
                    RequestType::TvDetails { .. } => self.response_tv_details(text),
                    RequestType::EpisodeDetails { id } => self.response_episode_details(text, id),
                    // is handled by the `Message::RequestPoster` case
                    RequestType::Poster { .. } => Command::none(),
                };
                update = StateUpdate::new(cmd).into();
            }
            Message::RequestPoster(id, handle) => {
                let Some(handle) = handle else {
                    warn!("No handle for poster submitted. MovieId {}", id);
                    return StateUpdate::default();
                };
                self.movie_posters.insert(id, Poster::Image(handle));
            }
            Message::FilterChanged(new_filter) => {
                debug!("changed filter from {:?} to {:?}", self.filter, new_filter);
                self.filter = new_filter;
                // Load the current episode details if not already loaded
                if let Filter::Details(movie_id) = new_filter {
                    self.set_detail_input_caches(movie_id);
                    let Some(bookmark) = self.get_bookmark(movie_id) else {
                        return StateUpdate::default();
                    };
                    let current_episode = bookmark.current_episode_id();

                    if !self.episode_details.contains_key(&current_episode) {
                        let msg = Message::ExecuteRequest(RequestType::EpisodeDetails {
                            id: current_episode,
                        });
                        let cmd = self.update_state(msg);
                        update = cmd.into();
                    }
                }
            }
            Message::AddBookmark(id) => {
                // We don't want to add the movie if we already have a bookmark for that movie
                if self.get_bookmark(id).is_some() {
                    info!("Ignore add bookmark message since a bookmark with that movie id already exists");
                    return StateUpdate::default();
                }
                let Some(movie) = self.movies.with_id(id) else {
                    warn!("Tried to add a bookmark for a movie, which is currently not loaded");
                    return StateUpdate::default();
                };
                self.bookmarks.push(Bookmark::from(movie));
            }
            Message::RemoveBookmark(id) => {
                let Some(index) = self.bookmarks.iter().position(|b| b.movie.id == id) else {
                    warn!(
                        "Tried to remove bookmark with {}, but no such bookmark exists",
                        id,
                    );
                    return StateUpdate::default();
                };
                debug!("Remove bookmark {:?}", &self.bookmarks[index]);
                self.bookmarks.remove(index);
                if Filter::Details(id) == self.filter {
                    let cmd = self.update_state(Message::FilterChanged(Filter::Bookmarks));
                    update = cmd.into();
                }
            }
            Message::BookmarkMessage(id, message) => {
                if let Some(bookmark) = self.bookmarks.with_id_mut(id) {
                    let cmd = bookmark.apply(message);
                    let current_episode = bookmark.current_episode_id();
                    let new_update = if !self.episode_details.contains_key(&current_episode) {
                        let msg = Message::ExecuteRequest(RequestType::EpisodeDetails {
                            id: current_episode,
                        });
                        let state = self.update_state(msg);
                        state.add_command(cmd)
                    } else {
                        StateUpdate::new(cmd)
                    };
                    update = new_update.into();
                } else {
                    warn!("bookmark message received, that couldn't be applied. Mes: {message:?} movie_id: {id} Bookmarks: {bookmarks:?}",message=message, id=id,bookmarks=&self.bookmarks);
                }
            }
            Message::LinkMessage(id, message) => {
                let Some(bookmark) = self.bookmarks.with_id_mut(id) else {
                    warn!(
                        "couldn't find bookmark which corresponds to link at position {}",
                        id
                    );
                    return StateUpdate::default();
                };
                let cmd = match message {
                    LinkMessage::LinkToClipboard(details, shift) => {
                        let Some(link) = self.links.with_id_mut(id) else {
                            warn!("couldn't find link at position {}", id);
                            return StateUpdate::default();
                        };
                        link.to_clipboard(bookmark, details, shift)
                    }
                };
                update = StateUpdate::new(cmd).into();
            }
            Message::TabPressed => {
                let cmd = if self.shift_pressed == ShiftPressed::True {
                    widget::focus_previous()
                } else {
                    widget::focus_next()
                };
                update = StateUpdate::new(cmd).into();
            }
            Message::ShiftPressed(shift) => self.shift_pressed = shift,
            Message::ToggleFullscreen(mode) => {
                let cmd = window::change_mode(window::Id::MAIN, mode);
                update = StateUpdate::new(cmd).into();
            }
        };
        update.unwrap_or_default()
    }
    fn config(&self) -> TmdbConfig {
        self.tmdb_config.clone().expect("TMDB config is not loaded")
    }
}
