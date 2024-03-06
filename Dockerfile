FROM rust:1.70 as build

WORKDIR /app

ADD src /app/src
ADD Cargo.toml /app/
ADD Cargo.lock /app/

RUN RUST_BACKTRACE=1 cargo build --release

ENV RUST_BACKTRACE=full

EXPOSE 8080
CMD ["./target/release/webhook-proxy"]
