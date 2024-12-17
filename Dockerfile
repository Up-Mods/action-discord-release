FROM rust:1.83.0 AS base

RUN cargo install cargo-chef --version ^0.1

WORKDIR /app

ARG RUST_ENVIRONMENT
ENV RUST_ENVIRONMENT=${RUST_ENVIRONMENT:-production}

FROM base AS planner

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef prepare --recipe-path recipe.json

FROM base AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --recipe-path recipe.json

ARG VERSION
ENV VERSION=${VERSION:-dev}

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bins

FROM gcr.io/distroless/cc AS runtime

ARG RUST_LOG
ENV RUST_LOG=${RUST_LOG:-info}

COPY --from=builder /app/target/release/action .

ENTRYPOINT ["/action"]
