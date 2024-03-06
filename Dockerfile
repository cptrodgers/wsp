## BUILD
FROM rust:1.70 as build

WORKDIR /app

ADD src /app/src
ADD Cargo.toml /app/
ADD Cargo.lock /app/

RUN RUST_BACKTRACE=1 cargo build --release

## RUNTIME ENVIRONMENT
FROM ubuntu:20.04
RUN apt-get update && apt-get -y upgrade && \
    apt-get -y install libpq-dev openssl ca-certificates libssl-dev

WORKDIR /app
COPY --from=build-phase /app/target/release/webhook-proxy /app

EXPOSE 8080
CMD ["./webhook-proxy"]
