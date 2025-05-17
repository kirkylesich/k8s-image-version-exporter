FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:stable-slim
COPY --from=builder /usr/src/app/target/release/k8s-image-version-exporter /usr/local/bin/k8s-image-version-exporter
EXPOSE 8080
ENTRYPOINT ["k8s-image-version-exporter"]
