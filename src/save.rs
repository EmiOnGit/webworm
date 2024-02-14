use std::{fs::File, io::Write};

use iced::widget::image::{self, Handle};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    bookmark::Bookmark,
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

impl SavedState {
    fn with_tmdb(mut self, tmdb_config: Option<TmdbConfig>) -> Self {
        self.tmdb_config = tmdb_config;
        self
    }
    fn path() -> std::path::PathBuf {
        let mut path =
            if let Some(project_dirs) = directories_next::ProjectDirs::from("", "", "Webworm") {
                project_dirs.data_dir().into()
            } else {
                std::env::current_dir().unwrap_or_default()
            };

        path.push("state.json");

        path
    }
    pub async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)
            .map_err(trace_io_error)?;
        let tmdb_config = TmdbConfig::new().await.ok();
        serde_json::from_str::<SavedState>(&contents)
            .map_err(|_| LoadError::Format)
            .map_err(trace_io_error)
            .map(|state| state.with_tmdb(tmdb_config))
    }

    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let path = Self::path();

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
    let mut path = if let Some(dirs) = directories_next::ProjectDirs::from("", "", "Webworm") {
        dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or_default()
    };
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
