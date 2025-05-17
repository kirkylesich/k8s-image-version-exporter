use k8s_image_version_exporter::ghcr::fetch_latest_tag;
use reqwest::Client;
use mockito::Server;

#[tokio::test]
async fn test_fetch_latest_tag_ghcr_token() {
    let mut server = Server::new_async().await;
    let body = r#"{"tags": ["0.1", "0.2"]}"#;
    let m = server
        .mock("GET", "/v2/siderolabs/flannel/tags/list")
        .match_header("authorization", "Bearer token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;
    std::env::set_var("GHCR_TOKEN", "token");
    let client = Client::builder().build().unwrap();
    let repo = format!("{}/siderolabs/flannel", server.url());
    let tag = fetch_latest_tag(&client, &repo).await.unwrap();
    m.assert_async().await;
    std::env::remove_var("GHCR_TOKEN");
    assert_eq!(tag, "0.2");
}

#[tokio::test]
async fn test_fetch_latest_tag_ghcr_no_token() {
    let mut server = Server::new_async().await;
    let body = r#"{"tags": ["1", "2"]}"#;
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
    assert_eq!(tag, "2");
}
