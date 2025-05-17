mod types;
mod dockerhub;
mod kube_watcher;
mod metrics;

use std::sync::Arc;
use dockerhub::fetch_latest_tag;
use kube_watcher::list_images;
use metrics::Metrics;
use kube::Client;
use reqwest::Client as HttpClient;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let k8s_client = Client::try_default().await?;
    let http_client = HttpClient::new();
    let metrics = Arc::new(Metrics::new());

    // spawn http server
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let make_svc = make_service_fn(move |_| {
            let m = metrics_clone.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |_req: Request<Body>| {
                    let m = m.clone();
                    async move {
                        let body = m.gather();
                        Ok::<_, hyper::Error>(Response::new(Body::from(body)))
                    }
                }))
            }
        });
        let addr = ([0, 0, 0, 0], 8080).into();
        Server::bind(&addr).serve(make_svc).await.unwrap();
    });

    loop {
        let images = list_images(k8s_client.clone()).await?;
        let mut outdated = Vec::new();
        for img in images {
            let repo = if img.repository.contains('/') {
                img.repository.clone()
            } else {
                format!("library/{}", img.repository)
            };
            match fetch_latest_tag(&http_client, &repo).await {
                Ok(latest) => {
                    if latest != img.tag {
                        outdated.push((repo, img.tag.clone(), latest));
                    }
                }
                Err(e) => {
                    eprintln!("error fetching latest tag for {}: {e}", repo);
                }
            }
        }
        metrics.set_outdated(&outdated);
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
