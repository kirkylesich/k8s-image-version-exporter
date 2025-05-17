use std::collections::HashSet;
use kube::{Api, Client};
use kube::api::ListParams;
use k8s_openapi::api::core::v1::Pod;
use crate::types::ImageRef;

pub async fn list_images(client: Client) -> Result<HashSet<ImageRef>, kube::Error> {
    let pods: Api<Pod> = Api::all(client);
    let lp = ListParams::default();
    let pod_list = pods.list(&lp).await?;
    let mut images = HashSet::new();
    for p in pod_list.items {
        if let Some(spec) = p.spec {
            for c in spec.containers.iter() {
                if let Some(image) = &c.image {
                    if let Some(img) = ImageRef::parse(image) {
                        images.insert(img);
                    }
                }
            }
        }
    }
    Ok(images)
}
