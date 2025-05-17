FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY vendor ./vendor
COPY src ./src
RUN cargo build --release && strip target/release/k8s-image-version-exporter

FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/k8s-image-version-exporter /usr/local/bin/k8s-image-version-exporter
EXPOSE 8080
ENTRYPOINT ["k8s-image-version-exporter"]
