use std::{convert::Infallible, sync::Arc};

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{rt::{TokioExecutor, TokioIo}, server::conn::auto::Builder};
use kube::Client;
use reqwest::Client as HttpClient;
use tokio::net::TcpListener;

use k8s_image_version_exporter::dockerhub::fetch_latest_tag as fetch_dockerhub_tag;
use k8s_image_version_exporter::registry::fetch_latest_tag as fetch_registry_tag;
use k8s_image_version_exporter::ghcr::fetch_latest_tag as fetch_ghcr_tag;
use k8s_image_version_exporter::kube_watcher::list_images;
use k8s_image_version_exporter::metrics::Metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let k8s_client = Client::try_default().await?;
    let http_client = HttpClient::new();
    let metrics = Arc::new(Metrics::new());

    // spawn http server
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, 8080)).await.unwrap();
        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("failed to accept connection: {e}");
                    continue;
                }
            };
            let m = metrics_clone.clone();
            tokio::spawn(async move {
                let service = service_fn(move |_req: Request<Incoming>| {
                    let m = m.clone();
                    async move {
                        let body = m.gather();
                        Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body)).boxed()))
                    }
                });
                if let Err(e) = Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(stream), service)
                    .await
                {
                    eprintln!("error serving {addr}: {e}");
                }
            });
        }
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
            let fetch_res = if repo.starts_with("ghcr.io/") || repo.starts_with("https://ghcr.io/") || repo.starts_with("http://ghcr.io/") {
                fetch_ghcr_tag(&http_client, &repo).await.map_err(|e| e.to_string())
            } else if repo.split('/').next().map_or(false, |p| p.contains('.')) {
                fetch_registry_tag(&http_client, &repo).await.map_err(|e| e.to_string())
            } else {
                fetch_dockerhub_tag(&http_client, &repo).await.map_err(|e| e.to_string())
            };
            match fetch_res {
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
