FROM rust:1.94-bookworm AS builder
WORKDIR /app

COPY camagru ./
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/camagru /usr/local/bin/camagru

EXPOSE 8080
CMD ["camagru"]
