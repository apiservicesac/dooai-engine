FROM rust:1-trixie AS builder
WORKDIR /project
COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY crates crates
RUN cargo build --release -p engine-server

FROM debian:trixie-slim
RUN useradd --system --uid 10001 engine
COPY --from=builder /project/target/release/engine-server /usr/local/bin/engine-server
USER engine
# The OCR models are not baked in: mount the output of scripts/download-models.sh at /models.
ENV ENGINE_MODELS_DIR=/models
EXPOSE 8080
ENTRYPOINT ["engine-server"]
