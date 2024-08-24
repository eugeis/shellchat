FROM rust:bookworm AS builder

WORKDIR /app

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./prompts.yaml ./prompts.yaml

RUN cargo build --release --bins


FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl

RUN apt-get update && apt-get install -y ca-certificates

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/shc-serve ./
COPY --from=builder /app/target/release/shc ./

CMD ["./shc-serve"]