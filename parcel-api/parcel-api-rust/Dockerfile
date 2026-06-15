# syntax=docker/dockerfile:1.7
#
# Build context is the parent directory (parcel-api/), so paths below are
# relative to that. Build with:
#   docker build --load -t parcel-api-rust -f parcel-api-rust/Dockerfile ../
# (run from parcel-api/parcel-api-rust/)
#
# Multi-stage build using cargo-chef so the dependency-compilation layer is
# cached separately from the application code. As long as Cargo.toml /
# Cargo.lock don't change, the heavy `cargo chef cook` layer is reused via
# normal docker layer caching, and rebuilds only recompile our own crate.
# That mirrors JVM's `gradle clean` semantics — deps stay cached, app code
# is rebuilt — so the build-time column in the bench is a fair comparison.

FROM rust:1-slim AS chef
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked --version "^0.1"

FROM chef AS planner
COPY parcel-api-rust/Cargo.toml parcel-api-rust/Cargo.lock* parcel-api-rust/rust-toolchain.toml ./
COPY parcel-api-rust/src ./src
COPY parcel-api-rust/tests ./tests
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# bench.sh passes --build-arg APP_BUILD_ID=$(date +%s) to invalidate just this
# stage's layers, forcing a fresh app-code compile while keeping the cached
# dep layer. Without the arg, repeated builds with unchanged source short-circuit.
ARG APP_BUILD_ID=stable
COPY parcel-api-rust/Cargo.toml parcel-api-rust/Cargo.lock* parcel-api-rust/rust-toolchain.toml ./
COPY parcel-api-rust/src ./src
COPY parcel-api-rust/tests ./tests
COPY parcel-api-rust/config ./config
RUN echo "build id ${APP_BUILD_ID}" \
    && cargo build --release --locked \
    && cp target/release/parcel-api /parcel-api

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app

COPY --from=builder /parcel-api /usr/local/bin/parcel-api
COPY parcel-api-rust/config /app/config
COPY parcel-data /parcel-data

ENV TZ=Europe/Oslo \
    APP_PROFILE=production \
    APP_CONFIG_DIR=/app/config \
    APP_PARCEL_DATA_DIR=/parcel-data

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/parcel-api"]
