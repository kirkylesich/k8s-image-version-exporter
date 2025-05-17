use k8s_image_version_exporter::kube_watcher::list_images;
use k8s_image_version_exporter::types::ImageRef;
use http::{Request, Response, StatusCode};
use hyper::Body;
use kube::{Client, Config};
use url::Url;
use tower_test::mock::Mock;

#[tokio::test]
async fn test_list_images() {
    let pod_list = serde_json::json!({
        "items": [
            {
                "spec": {
                    "containers": [
                        {"image": "nginx:1.1"},
                        {"image": "busybox"}
                    ]
                }
            }
        ]
    });

    let (service, mut handle) = Mock::pair();

    tokio::spawn(async move {
        if let Some(req) = handle.next_request().await {
            assert_eq!(req.uri().path(), "/api/v1/pods");
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(pod_list.to_string()))
                .unwrap();
            req.response.send_response(resp);
        }
    });

    let config = Config {
        cluster_url: Url::parse("http://localhost").unwrap(),
        default_namespace: "default".into(),
        root_cert: None,
        accept_invalid_certs: true,
        auth_info: Default::default(),
        ..Default::default()
    };
    let client = Client::new(service, config);

    let images = list_images(client).await.expect("images");
    assert!(images.contains(&ImageRef { repository: "nginx".into(), tag: "1.1".into() }));
    assert!(images.contains(&ImageRef { repository: "busybox".into(), tag: "latest".into() }));
}
