# Multi-stage build: compile in a full Rust image, run in a slim Debian image.

FROM rust:1-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
# Cargo validates every [[test]] path while parsing the manifest, even when only
# building a single --bin target below, so tests/ must be present in the build context.
COPY tests ./tests

# --bin skips compiling the bdd test binary and its dev-dependencies (cucumber,
# testcontainers, reqwest), which aren't needed in the production image.
RUN cargo build --release --bin elevia-api

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/elevia-api ./elevia-api
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080
CMD ["./elevia-api"]
