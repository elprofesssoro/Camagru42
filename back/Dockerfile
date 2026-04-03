FROM rust:1.94-bookworm AS builder
WORKDIR /app

COPY camagru ./
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/camagru /usr/local/bin/camagru

EXPOSE 8080
CMD ["camagru"]
