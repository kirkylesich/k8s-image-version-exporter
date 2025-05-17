use k8s_image_version_exporter::kube_watcher::list_images;
use k8s_image_version_exporter::types::ImageRef;
use http::{Request, Response, StatusCode};
use kube::client::Body;
use kube::Client;
use tower_test::mock;

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

    let (service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();

    tokio::spawn(async move {
        if let Some((req, send)) = handle.next_request().await {
            assert_eq!(req.uri().path(), "/api/v1/pods");
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(pod_list.to_string().into_bytes()))
                .unwrap();
            send.send_response(resp);
        }
    });

    let client = Client::new(service, "default");

    let images = list_images(client).await.expect("images");
    assert!(images.contains(&ImageRef { repository: "nginx".into(), tag: "1.1".into() }));
    assert!(images.contains(&ImageRef { repository: "busybox".into(), tag: "latest".into() }));
}

#[tokio::test]
async fn test_list_images_handles_missing_spec_and_image() {
    let pod_list = serde_json::json!({
        "items": [
            {},
            {
                "spec": {
                    "containers": [
                        {"image": null},
                        {}
                    ]
                }
            }
        ]
    });

    let (service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();

    tokio::spawn(async move {
        if let Some((req, send)) = handle.next_request().await {
            assert_eq!(req.uri().path(), "/api/v1/pods");
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(pod_list.to_string().into_bytes()))
                .unwrap();
            send.send_response(resp);
        }
    });

    let client = Client::new(service, "default");

    let images = list_images(client).await.expect("images");
    assert!(images.is_empty());
}

#[tokio::test]
async fn test_list_images_error_response() {
    let (service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();

    tokio::spawn(async move {
        if let Some((req, send)) = handle.next_request().await {
            assert_eq!(req.uri().path(), "/api/v1/pods");
            let resp = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap();
            send.send_response(resp);
        }
    });

    let client = Client::new(service, "default");

    let res = list_images(client).await;
    assert!(res.is_err());
}
