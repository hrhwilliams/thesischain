FROM rust:1.91-slim-trixie AS chef
WORKDIR /build
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
WORKDIR /build
RUN apt update && apt install -y libpq-dev openssl libssl-dev pkg-config
RUN cargo install diesel_cli --no-default-features --features postgres
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:trixie-slim AS runtime
RUN apt-get  update && apt-get install -y postgresql
WORKDIR /app
COPY --from=backend-builder /build/target/release/end2 .
COPY --from=backend-builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=backend-builder /build/migrations /app/migrations
COPY --from=backend-builder /build/entrypoint.sh .
ENTRYPOINT [ "./entrypoint.sh" ]
CMD ["./end2"]