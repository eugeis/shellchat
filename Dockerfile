FROM rust:bookworm AS builder

WORKDIR /app

COPY ./ .

RUN cargo build --release --bins


FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl

RUN apt-get update && apt-get install -y ca-certificates

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/sc-serve ./
COPY --from=builder /app/target/release/sc ./


CMD ["./sc-serve"]