use iced::Command;
use tracing::{error, info};

use crate::{
    id::EpisodeId,
    message::Message,
    movie_details::{Episode, EpisodeDetails, MovieDetails},
    state::State,
    tmdb::{RequestType, TmdbResponse},
};

impl State {
    pub fn response_tv_search(&mut self, text: String) -> Command<Message> {
        let response: serde_json::Result<TmdbResponse> = serde_json::from_str(&text);
        let Ok(response) = response else {
            error!("Failed to parse tv search with: {response:?}");
            return Command::none();
        };
        self.movies = response.results;
        let mut cmds = Vec::new();
        for movie in self.movies.clone() {
            let id = movie.id;
            let msg = Message::ExecuteRequest(RequestType::TvDetails { id });
            let cmd = self.update_state(msg).command();
            cmds.push(cmd);
            if let Some(path) = movie.poster_path.clone() {
                let msg = Message::ExecuteRequest(RequestType::Poster { id, path });
                let cmd = self.update_state(msg).command();
                cmds.push(cmd);
            }
        }
        Command::batch(cmds)
    }
    pub fn response_tv_details(&mut self, text: String) -> Command<Message> {
        let response: serde_json::Result<MovieDetails> = serde_json::from_str(&text);
        let Ok(mut response) = response else {
            error!("Failed to parse tv details with: {response:?}");
            return Command::none();
        };
        response.fix_episode_formats();
        if let Some(bookmark) = self
            .bookmarks
            .iter_mut()
            .find(|bookmark| bookmark.movie.id == response.id)
        {
            if let Episode::Total(e) = &bookmark.current_episode {
                bookmark.current_episode = response.as_seasonal_episode(e).into();
            }
            if bookmark.finished {
                let next = response.next_episode(bookmark.current_episode.clone());
                if next != bookmark.current_episode {
                    info!("Found new episode for {:?}. Reset finished state", bookmark);
                    bookmark.finished = false;
                    bookmark.current_episode = next;
                }
            }
        }
        self.movie_details.insert(response.id, response);
        Command::none()
    }
    pub fn response_episode_details(&mut self, text: String, id: EpisodeId) -> Command<Message> {
        let response: serde_json::Result<EpisodeDetails> = serde_json::from_str(&text);
        let Ok(response) = response else {
            error!("Failed to parse episode details with: {response:?}");
            return Command::none();
        };
        self.episode_details.insert(id, response);
        Command::none()
    }
}
