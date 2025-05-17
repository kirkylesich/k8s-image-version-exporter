use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GhcrError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("no tags found for repo")]
    NoTags,
    #[error("invalid repository")]
    InvalidRepo,
}

#[derive(Debug, Deserialize)]
struct TagsList {
    tags: Option<Vec<String>>,
}

pub async fn fetch_latest_tag(client: &Client, repo: &str) -> Result<String, GhcrError> {
    let (scheme, rest) = if let Some(stripped) = repo.strip_prefix("http://") {
        ("http", stripped)
    } else if let Some(stripped) = repo.strip_prefix("https://") {
        ("https", stripped)
    } else {
        ("https", repo)
    };
    let (host, path) = rest.split_once('/') .ok_or(GhcrError::InvalidRepo)?;
    let url = format!("{scheme}://{host}/v2/{path}/tags/list");
    let mut req = client.get(url);
    if let Ok(token) = std::env::var("GHCR_TOKEN") {
        req = req.bearer_auth(token);
    }
    let resp: TagsList = req.send().await?.json().await?;
    let tags = resp.tags.ok_or(GhcrError::NoTags)?;
    if tags.is_empty() {
        return Err(GhcrError::NoTags);
    }
    Ok(determine_latest_tag(tags))
}

fn determine_latest_tag(mut tags: Vec<String>) -> String {
    tags.sort_by(|a, b| compare_versions(a, b));
    tags.pop().unwrap()
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let va = parse_version(a);
    let vb = parse_version(b);
    match (&va, &vb) {
        (Some(ref av), Some(ref bv)) => cmp_semver(av, bv),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

fn parse_version(tag: &str) -> Option<Vec<u64>> {
    let t = tag.trim_start_matches('v');
    if t.chars().all(|c| c.is_ascii_digit() || c == '.') {
        let parts: Vec<u64> = t
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect();
        Some(parts)
    } else {
        None
    }
}

fn cmp_semver(a: &Vec<u64>, b: &Vec<u64>) -> std::cmp::Ordering {
    let len = std::cmp::max(a.len(), b.len());
    for i in 0..len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        match av.cmp(&bv) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_latest_tag_ghcr_no_token() {
        let mut server = Server::new_async().await;
        let body = r#"{"tags": ["1.0", "1.2"]}"#;
        let m = server
            .mock("GET", "/v2/siderolabs/flannel/tags/list")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;
        let client = Client::builder().build().unwrap();
        let repo = format!("{}/siderolabs/flannel", server.url());
        let tag = fetch_latest_tag(&client, &repo).await.unwrap();
        m.assert_async().await;
        assert_eq!(tag, "1.2");
    }

    #[tokio::test]
    async fn test_fetch_latest_tag_ghcr_with_token() {
        let mut server = Server::new_async().await;
        let body = r#"{"tags": ["v2.0", "v2.1"]}"#;
        let m = server
            .mock("GET", "/v2/siderolabs/flannel/tags/list")
            .match_header("authorization", "Bearer secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;
        std::env::set_var("GHCR_TOKEN", "secret");
        let client = Client::builder().build().unwrap();
        let repo = format!("{}/siderolabs/flannel", server.url());
        let tag = fetch_latest_tag(&client, &repo).await.unwrap();
        m.assert_async().await;
        std::env::remove_var("GHCR_TOKEN");
        assert_eq!(tag, "v2.1");
    }
}

