# Stage 1: Build web UI
FROM node:20-slim AS web-builder
WORKDIR /app/web-ui
COPY web-ui/package.json web-ui/package-lock.json ./
RUN npm ci
COPY web-ui/ ./
RUN npx vite build

# Stage 2: Build Rust binary
FROM rust:1.85-slim AS rust-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY --from=web-builder /app/web-ui/dist/ web-ui/dist/
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /app/target/release/folio /usr/local/bin/folio
EXPOSE 8080
CMD ["folio", "serve", "--host", "0.0.0.0", "--port", "8080"]
