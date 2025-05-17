use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerHubError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("no tags found for repo")] 
    NoTags,
}

#[derive(Debug, Deserialize)]
struct TagResult {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    results: Vec<TagResult>,
}

pub async fn fetch_latest_tag(client: &Client, repo: &str) -> Result<String, DockerHubError> {
    // repo should be in form "library/nginx" or "user/repo"
    let url = format!(
        "https://hub.docker.com/v2/repositories/{repo}/tags?page_size=1&ordering=last_updated"
    );
    let resp: TagsResponse = client.get(url).send().await?.json().await?;
    resp.results
        .into_iter()
        .next()
        .map(|t| t.name)
        .ok_or(DockerHubError::NoTags)
}
