use std::{fs::File, io::Write};

use iced::{
    widget::image::{self, Handle},
    Command,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    bookmark::Bookmark,
    gui::App,
    message::Message,
    state::State,
    tmdb::{self, RequestType, TmdbConfig},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub bookmarks: Vec<Bookmark>,
    #[serde(skip)]
    pub tmdb_config: Option<TmdbConfig>,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    File,
    Format,
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
        std::env::current_dir().unwrap_or_default()
    }
}
impl SavedState {
    fn with_tmdb(mut self, tmdb_config: Option<TmdbConfig>) -> Self {
        self.tmdb_config = tmdb_config;
        self
    }
    pub async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();
        let mut conf_contents = String::new();
        let path = path();
        let mut state_path = path.clone();
        state_path.push("state.json");
        let mut conf_path = path.clone();
        conf_path.push("cred");

        let mut state_file = async_std::fs::File::open(state_path)
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;

        state_file
            .read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;
        let mut cred_file = async_std::fs::File::open(conf_path)
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;

        cred_file
            .read_to_string(&mut conf_contents)
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;

        let tmdb_config = TmdbConfig::new(&conf_contents);
        serde_json::from_str::<SavedState>(&contents)
            .map_err(|_| LoadError::Format)
            .map_err(trace_io_error)
            .map(|state| state.with_tmdb(tmdb_config))
    }

    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let path = path();

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)
                .map_err(trace_io_error)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::File)
                .map_err(trace_io_error)?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| SaveError::Write)
                .map_err(trace_io_error)?;
        }

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}
fn trace_io_error<T: std::fmt::Debug>(t: T) -> T {
    error!("Saving/Loading failed with {t:?}");
    t
}
pub async fn load_poster(id: usize, url: String, config: TmdbConfig) -> anyhow::Result<Handle> {
    let mut path = path();
    path.push("posters");
    path.push(format!("{}.png", id));
    if path.exists() {
        let bytes = async_std::fs::read(path).await?;
        let handle = image::Handle::from_memory(bytes);
        Ok(handle)
    } else {
        let req = RequestType::Poster { id, path: url };
        let response = tmdb::send_byte_request(config.clone(), req).await?;
        File::create(path)?.write_all(&response)?;
        let handle = image::Handle::from_memory(response);
        Ok(handle)
    }
}
impl App {
    pub fn as_loaded(&mut self, state: SavedState) -> Command<Message> {
        // set self to be loaded
        *self = App::Loaded(State {
            tmdb_config: state.tmdb_config,
            bookmarks: state.bookmarks.clone(),
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
                if let Some(poster_path) = &bookmark.movie.poster_path {
                    Some(RequestType::Poster {
                        id: bookmark.movie.id,
                        path: poster_path.clone(),
                    })
                } else {
                    None
                }
            })
            .map(|req| {
                Command::perform(async { Ok(()) }, |_: Result<(), ()>| {
                    Message::ExecuteRequest(req)
                })
            });

        Command::batch(iter_load_details.chain(iter_load_posters))
    }
}
