FROM rust:1.91-slim-trixie AS chef
WORKDIR /build
RUN apt-get update && apt-get install -y clang libclang-dev cmake && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

FROM chef AS planner
COPY ./crates/end2-cometbft ./end2-cometbft
RUN echo '[workspace]\nmembers = ["end2-cometbft"]\nresolver = "3"\n[workspace.package]\nedition = "2024"\n' > Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./crates/end2-cometbft /build/end2-cometbft
RUN cargo build --release

FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /build/target/release/end2-cometbft .
CMD ["./end2-cometbft"]
