# K8s Image Version Exporter

A small utility that scans running Kubernetes pods, compares container image tags with the latest tags on Docker Hub, and exposes metrics for Prometheus.

## Features
- Fetches list of images from all pods in the cluster
- Queries Docker Hub for the most recent tag of each image
- Publishes `image_outdated` metric with repository, current and latest tag
- Exposes metrics over HTTP on port `8080`

## Building

### Using Cargo
```bash
cargo build --release
```

The binary will be available at `target/release/k8s-image-version-exporter`.

### Using Docker
```bash
docker build -t kirill02102/k8s-image-version-exporter .
docker push kirill02102/k8s-image-version-exporter
```

## Running

The exporter needs access to the Kubernetes API. Ensure your kubeconfig is set or run inside a cluster with the appropriate ServiceAccount.

```bash
cargo run --release
```

Metrics will be available at `http://localhost:8080/`.

To run via Docker:
```bash
docker run --rm -p 8080:8080 -v $HOME/.kube:/root/.kube k8s-image-version-exporter
```

### Kubernetes Deployment

The repository includes manifests under `deploy/` to run the exporter inside a Kubernetes cluster.

```bash
kubectl apply -f deploy/k8s.yaml
```

The deployment uses a dedicated `ServiceAccount` and RBAC permissions so the
exporter can list pods across the cluster.

## Testing

Run the Rust test suite with:
```bash
cargo test
```

## Metrics

The exporter exposes a single Prometheus metric:

```
# HELP image_outdated Container images with outdated tag
# TYPE image_outdated gauge
image_outdated{repository="<repo>",current="<current_tag>",latest="<latest_tag>"} 1
```

Use this metric to alert on pods that are not running the latest image tag.

