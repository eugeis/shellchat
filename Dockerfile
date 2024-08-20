FROM debian:buster-slim

WORKDIR /usr/local/bin

COPY target/release/sc-serve .

CMD ["./sc-serve"]