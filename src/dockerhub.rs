use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::registry::{determine_latest_tag, parse_version};

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
    // repo can be in form "library/nginx" or a full url to the repository
    let url = if repo.starts_with("http://") || repo.starts_with("https://") {
        format!("{repo}/tags?page_size=100&ordering=last_updated")
    } else {
        format!(
            "https://hub.docker.com/v2/repositories/{repo}/tags?page_size=100&ordering=last_updated"
        )
    };
    let resp: TagsResponse = client.get(url).send().await?.json().await?;
    let tags: Vec<String> = resp.results.into_iter().map(|t| t.name).collect();
    if tags.is_empty() {
        return Err(DockerHubError::NoTags);
    }
    let numeric: Vec<String> = tags
        .iter()
        .filter(|t| parse_version(t).is_some())
        .cloned()
        .collect();
    if !numeric.is_empty() {
        Ok(determine_latest_tag(numeric))
    } else {
        Ok(tags[0].clone())
    }
}
