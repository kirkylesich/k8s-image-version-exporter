use k8s_image_version_exporter::dockerhub::fetch_latest_tag;
use mockito::Server;
use reqwest::Client;

#[tokio::test]
async fn test_fetch_latest_tag() {
    let mut server = Server::new_async().await;
    let body = r#"{"results": [{"name": "1.2.3"}]}"#;
    let m = server
        .mock("GET", "/v2/repositories/library/nginx/tags")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
            mockito::Matcher::UrlEncoded("ordering".into(), "last_updated".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = Client::builder()
        .default_headers(reqwest::header::HeaderMap::new())
        .build()
        .unwrap();

    let repo = format!("{}/v2/repositories/library/nginx", server.url());
    // patch fetch_latest_tag's URL by overriding repo
    let tag = fetch_latest_tag(&client, &repo).await.unwrap();
    m.assert_async().await;
    assert_eq!(tag, "1.2.3");
}

#[tokio::test]
async fn test_fetch_latest_tag_no_tags() {
    let mut server = Server::new_async().await;
    let body = r#"{"results": []}"#;
    let m = server
        .mock("GET", "/v2/repositories/library/nginx/tags")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
            mockito::Matcher::UrlEncoded("ordering".into(), "last_updated".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = Client::builder()
        .default_headers(reqwest::header::HeaderMap::new())
        .build()
        .unwrap();

    let repo = format!("{}/v2/repositories/library/nginx", server.url());
    let res = fetch_latest_tag(&client, &repo).await;
    m.assert_async().await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_fetch_latest_tag_plain_repo() {
    let mut server = Server::new_async().await;
    let body = r#"{"results": [{"name": "2.0"}]}"#;
    let m = server
        .mock("GET", "/v2/repositories/library/busybox/tags")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
            mockito::Matcher::UrlEncoded("ordering".into(), "last_updated".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = Client::builder()
        .default_headers(reqwest::header::HeaderMap::new())
        .build()
        .unwrap();

    let repo = format!("{}/v2/repositories/library/busybox", server.url());
    let tag = fetch_latest_tag(&client, &repo).await.unwrap();
    m.assert_async().await;
    assert_eq!(tag, "2.0");
}

#[tokio::test]
async fn test_fetch_latest_tag_filters_non_numeric() {
    let mut server = Server::new_async().await;
    let body = r#"{"results": [{"name": "stable-perl"}, {"name": "1.4"}]}"#;
    let m = server
        .mock("GET", "/v2/repositories/library/nginx/tags")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
            mockito::Matcher::UrlEncoded("ordering".into(), "last_updated".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = Client::builder()
        .default_headers(reqwest::header::HeaderMap::new())
        .build()
        .unwrap();

    let repo = format!("{}/v2/repositories/library/nginx", server.url());
    let tag = fetch_latest_tag(&client, &repo).await.unwrap();
    m.assert_async().await;
    assert_eq!(tag, "1.4");
}
