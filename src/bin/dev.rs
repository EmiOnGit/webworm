use anyhow::Result;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new().await?;
    let request = request(config, "one piece");
    println!("request: {request:?} \n");
    let response = Client::new().execute(request).await?;
    println!("response: {response:?}\n");
    let data = response.text().await?;
    println!("response: {data}");
    Ok(())
}
pub fn request(config: Config, query: &str) -> reqwest::Request {
    let query_clean = query.replace(' ', "%20");
    let base_url = "https://api.themoviedb.org/3/search/tv?";
    let rest = "language=en-US&page=1";
    let url = format!("{base_url}&query={query_clean}&{rest}");
    println!("url: {url}");
    println!("-----");

    let request = Client::new()
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(config.token)
        .build()
        .unwrap();
    request
}
#[derive(Debug)]
pub struct Config {
    token: String,
    _key: String,
}
impl Config {
    pub async fn new() -> Result<Config> {
        let content = async_std::fs::read_to_string("cred.md").await?;
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.starts_with("#"))
            .collect();
        Ok(Config {
            token: lines[0].to_owned(),
            _key: lines[1].to_owned(),
        })
    }
}
