FROM rust:1.46 as builder
MAINTAINER Mikkel Kroman <mk@maero.dk>

WORKDIR /usr/src/meta-matrix
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && \
  apt-get install -y openssl && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/meta-matrix /usr/local/bin/meta-matrix
ENTRYPOINT /usr/local/bin/meta-matrix
