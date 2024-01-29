use anyhow::Result;
use reqwest::blocking::Client;

pub async fn queue_tv_series(config: TmdbConfig, query: String) -> Result<String> {
    let request = request(&config, &query);

    let response = Client::new().execute(request)?;

    let data: String = response.text().unwrap();
    Ok(data)
}
pub fn request(config: &TmdbConfig, query: &str) -> reqwest::blocking::Request {
    let query_clean = query.replace(' ', "%20");
    let base_url = "https://api.themoviedb.org/3/search/tv?";
    let rest = "language=en-US&page=1";
    let url = format!("{base_url}&query={query_clean}&{rest}");

    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(config.token.clone())
        .build()
        .unwrap();
    request
}
#[derive(Debug, Clone)]
pub struct TmdbConfig {
    token: String,
    _key: String,
}
impl TmdbConfig {
    pub async fn new() -> Result<TmdbConfig> {
        let content = async_std::fs::read_to_string("cred.md").await?;
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.starts_with("#"))
            .collect();
        Ok(TmdbConfig {
            token: lines[0].to_owned(),
            _key: lines[1].to_owned(),
        })
    }
}
