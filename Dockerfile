FROM rust:1.90-slim-trixie AS chef
WORKDIR /build
RUN apt update -y \
    && apt install -y --no-install-recommends protobuf-compiler libssl-dev pkg-config \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --package blockchain

FROM debian:trixie-slim AS runtime
WORKDIR /app
RUN apt update -y \
    && apt install -y --no-install-recommends openssl ca-certificates \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/blockchain blockchain
COPY --from=builder /build/blockchain ./src/blockchain
COPY --from=builder /build/core ./src/core
CMD ["sleep", "infinity"]