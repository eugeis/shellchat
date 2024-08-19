WORKDIR /app

# Copy the source code
COPY . .

# Build the server
RUN cargo build --release --bin sc-server

# Production stage
FROM debian:buster-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/sc-server .

CMD ["./sc-server"]