FROM rust:1.70 AS builder
WORKDIR /workdir                       
ENV CARGO_HOME=/workdir/.cargo                       
COPY ./Cargo.toml ./Cargo.lock ./                       
# Dummy build to cache deps
RUN mkdir src && touch src/lib.rs
RUN cargo build --release


COPY ./src ./src
RUN cargo build --release

FROM ubuntu:jammy

# RUN apt-get update && apt-get install procps
# RUN sysctl -w net.ipv6.bindv6only=1

# RUN echo "net.ipv6.bindv6only=1" >> /etc/sysctl.conf

EXPOSE 8080
COPY --from=builder /workdir/target/release/reflect /usr/local/bin

# ENTRYPOINT ["/usr/sbin/sysctl -w net.ipv6.bindv6only=1", "/usr/local/bin/reflect"]
ENTRYPOINT ["/usr/local/bin/reflect"]