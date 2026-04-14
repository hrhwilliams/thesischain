FROM rust:1.91-slim-trixie AS chef
WORKDIR /build
RUN cargo install cargo-chef

# ── Planner ───────────────────────────────────────────────────────────────────

FROM chef AS planner
COPY Cargo.lock ./
COPY crates/end2-cometbft ./crates/end2-cometbft
# Synthetic minimal workspace: only this crate + the workspace deps it inherits
RUN printf '[workspace]\nmembers = ["crates/end2-cometbft"]\nresolver = "3"\n\n[workspace.package]\nedition = "2024"\n\n[workspace.dependencies]\nbase64 = "0.22.1"\ndotenvy = "0.15.7"\n' > Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# ── Builder ───────────────────────────────────────────────────────────────────

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN printf '[workspace]\nmembers = ["crates/end2-cometbft"]\nresolver = "3"\n\n[workspace.package]\nedition = "2024"\n\n[workspace.dependencies]\nbase64 = "0.22.1"\ndotenvy = "0.15.7"\n' > Cargo.toml
COPY Cargo.lock ./
RUN cargo chef cook --release --recipe-path recipe.json
COPY crates/end2-cometbft ./crates/end2-cometbft
RUN cargo build --release -p end2-cometbft

# ── Runtime ───────────────────────────────────────────────────────────────────

FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /build/target/release/end2-cometbft .
CMD ["./end2-cometbft"]
