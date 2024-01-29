use serde::{Deserialize, Serialize};

use crate::{filter::Filter, task::TmdbMovie, tmdb::TmdbConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub input_value: String,
    pub filter: Filter,
    pub tasks: Vec<TmdbMovie>,
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
        let mut path = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("rs", "Iced", "Todos")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        path.push("todos.json");

        path
    }
    pub async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)?;
        let tmdb_config = TmdbConfig::new().await.ok();
        println!("conf: {:?}", tmdb_config);
        serde_json::from_str::<SavedState>(&contents)
            .map_err(|_| LoadError::Format)
            .map(|state| state.with_tmdb(tmdb_config))
    }

    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::File)?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| SaveError::Write)?;
        }

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}
