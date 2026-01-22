FROM rust:1.91-alpine3.22 AS wasm-builder
WORKDIR /build
RUN cargo install wasm-pack
COPY Cargo.toml .
COPY ./src ./src
RUN wasm-pack build --target bundler

FROM node:24-alpine AS frontend-builder
WORKDIR /app/frontend
COPY ./frontend /app/frontend
COPY --from=wasm-builder /build/pkg /app/pkg
RUN npm install
RUN node --run build

FROM frontend-builder AS frontend
WORKDIR /app/frontend
RUN npm install -g serve
CMD ["serve", "-s", "dist", "-l", "8081"]