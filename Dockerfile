FROM rust:1.70 AS builder
WORKDIR /workdir                       
ENV CARGO_HOME=/workdir/.cargo                       
COPY ./Cargo.toml ./Cargo.lock ./                       
# Dummy build to cache deps
RUN mkdir src && touch src/lib.rs
RUN cargo build --release

# Real build
COPY ./src ./src
RUN cargo build --release

FROM ubuntu:jammy

RUN apt-get update \
  && apt-get install net-tools curl --no-install-recommends -y \
  && rm -rf /var/lib/apt/lists/*

EXPOSE 8080
COPY --from=builder /workdir/target/release/reflect /usr/local/bin

ENTRYPOINT ["/usr/local/bin/reflect"]