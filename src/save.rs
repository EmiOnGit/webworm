use std::{collections::HashMap, fs::File, io::Write};

use async_std::{fs::create_dir_all, io::ReadExt};
use iced::{
    widget::image::{self, Handle},
    Command,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::{
    bookmark::Bookmark,
    gui::App,
    id::MovieId,
    link::Link,
    message::Message,
    state::State,
    tmdb::{self, RequestType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub bookmarks: Vec<Bookmark>,
    pub links: HashMap<MovieId, Link>,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    /// Happen if the state file exists but `File::open` results in a error
    OpenFile,
    /// Happen if the creation of the state file results in a error
    CreateFile,
    /// Happens if reading the file to a string results in an error
    ReadFile,
    /// Hapens if deserializing the state to the struct failed
    DeserializationError(String),
}

#[derive(Debug, Clone)]
pub enum SaveError {
    File,
    Write,
    Format,
}

fn path() -> std::path::PathBuf {
    if let Some(project_dirs) = directories_next::ProjectDirs::from("", "", "Webworm") {
        project_dirs.data_dir().into()
    } else {
        error!("Could not retrieve project directory. Using current directory instead");
        std::env::current_dir().unwrap_or_default()
    }
}
fn poster_path() -> std::path::PathBuf {
    let mut path = path();
    path.push("posters");
    path
}
impl SavedState {
    pub async fn load() -> Result<SavedState, LoadError> {
        let mut contents = String::new();
        let path = path();
        let mut state_path = path.clone();
        state_path.push("state.json");
        let mut state_file = if state_path.is_file() {
            async_std::fs::File::open(state_path).await.map_err(|e| {
                error!("failed to open state file with error {e:?}");
                LoadError::OpenFile
            })?
        } else {
            if let Ok(()) = create_dir_all(&path).await {
                warn!("created directories {path:?}");
            };
            async_std::fs::File::create(state_path).await.map_err(|e| {
                error!("failed to create a state file with error {e:?}");
                LoadError::CreateFile
            })?
        };

        state_file
            .read_to_string(&mut contents)
            .await
            .map_err(|e| {
                error!("failed to read content of state file to sting with error {e}");
                LoadError::ReadFile
            })?;

        serde_json::from_str::<SavedState>(&contents)
            .map_err(|e| LoadError::DeserializationError(e.to_string()))
    }

    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let mut path = path();
        path.push("state.json");

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)
                .map_err(trace_io_error)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(trace_io_error)
                .map_err(|_| SaveError::File)
                .map_err(trace_io_error)?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| SaveError::Write)
                .map_err(trace_io_error)?;
        }

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;

        Ok(())
    }
}
fn trace_io_error<T: std::fmt::Debug>(t: T) -> T {
    error!("Saving/Loading failed with {t:?}");
    t
}
pub async fn load_poster(id: MovieId, url: String) -> anyhow::Result<Handle> {
    let mut path = poster_path();
    if !path.exists() {
        let _ = create_dir_all(&path).await;
        warn!("created poster folder");
    }
    path.push(format!("{}.png", id));
    if path.exists() {
        let bytes = async_std::fs::read(path).await?;
        let handle = image::Handle::from_memory(bytes);
        Ok(handle)
    } else {
        let req = RequestType::Poster { id, path: url };
        let response = tmdb::send_byte_request(req).await?;
        File::create(path)?.write_all(&response)?;
        let handle = image::Handle::from_memory(response);
        Ok(handle)
    }
}
impl App {
    pub fn as_loaded(&mut self, state: SavedState) -> Command<Message> {
        // set self to be loaded
        *self = App::Loaded(State {
            bookmarks: state.bookmarks.clone(),
            links: state.links,
            ..State::default()
        });
        // load new data for the bookmarks
        let iter_load_details = state
            .bookmarks
            .iter()
            .map(|bookmark| RequestType::TvDetails {
                id: bookmark.movie.id,
            })
            .map(|req| {
                Command::perform(async { Ok(()) }, |_: Result<(), ()>| {
                    Message::ExecuteRequest(req)
                })
            });
        let iter_load_posters = state
            .bookmarks
            .iter()
            .filter_map(|bookmark| {
                bookmark
                    .movie
                    .poster_path
                    .as_ref()
                    .map(|poster_path| RequestType::Poster {
                        id: bookmark.movie.id,
                        path: poster_path.clone(),
                    })
            })
            .map(|req| {
                Command::perform(async { Ok(()) }, |_: Result<(), ()>| {
                    Message::ExecuteRequest(req)
                })
            });
        debug!("Finished loading the app state. Loading details and images next");
        Command::batch(iter_load_details.chain(iter_load_posters))
    }
}
