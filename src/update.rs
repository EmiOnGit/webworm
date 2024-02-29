use iced::{widget, window, Command};
use tracing::{debug, info, warn};

use crate::{
    bookmark::{Bookmark, Poster},
    filter::Filter,
    id::MovieIndex,
    link::BookmarkLinkBox,
    message::{LinkMessage, Message, ShiftPressed},
    save::load_poster,
    state::State,
    tmdb::{send_request, RequestType, TmdbConfig},
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
            Message::InputChanged(v) => self.input_value = v,
            Message::InputSubmit(input) => match self.filter {
                Filter::Search => {
                    let request = RequestType::TvSearch { query: input };
                    update = self.update_state(Message::ExecuteRequest(request)).into();
                }
                Filter::Details(_) => warn!("Input submit in details view received"),
                Filter::Bookmarks | Filter::Completed => {
                    info!("ignore input submit in current filter")
                }
            },

            Message::ExecuteRequest(request) => {
                let config = self.config();
                let cmd = if let RequestType::Poster { id, path } = request {
                    Command::perform(load_poster(id, path.clone(), config), move |data| {
                        Message::RequestPoster(id, data.ok())
                    })
                } else {
                    Command::perform(send_request(config, request.clone()), |data| {
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
                self.links
                    .insert(movie.id, BookmarkLinkBox::Input(String::new()));
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
            }
            Message::BookmarkMessage(id, message) => {
                if let Some(bookmark) = self.bookmarks.with_id_mut(id) {
                    let cmd = bookmark.apply(message);
                    update = StateUpdate::new(cmd).into();
                } else {
                    warn!("bookmark message received, that couldn't be applied. Mes: {message:?} movie_id: {id} Bookmarks: {bookmarks:?}",message=message, id=id,bookmarks=&self.bookmarks);
                }
            }
            Message::LinkMessage(id, mut message) => {
                if let LinkMessage::LinkToClipboard(_, ref mut shift) = message {
                    *shift = self.shift_pressed.clone();
                };
                let Some(bookmark) = self.bookmarks.with_id_mut(id) else {
                    warn!(
                        "couldn't find bookmark which corresponds to link at position {}",
                        id
                    );
                    return StateUpdate::default();
                };
                let Some(link) = self.links.with_id_mut(id) else {
                    warn!("couldn't find link at position {}", id);
                    return StateUpdate::default();
                };
                let cmd = link.apply(bookmark, message);
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
