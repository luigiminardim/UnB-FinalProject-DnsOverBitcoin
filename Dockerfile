FROM rust:1.80.0 AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY /src ./src
RUN cargo build --release

FROM debian:12.7
WORKDIR /app
COPY --from=builder /app/target/release/ordinals_dns ./nostr-dns-server
COPY data ./data
CMD ["./nostr-dns-server"]
