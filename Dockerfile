# Build stage

FROM rust:1.68-buster as cargo-build
RUN apt-get update && apt-get -y install libolm-dev cmake

WORKDIR /usr/src/notion2ics
COPY Cargo.lock .
COPY Cargo.toml .
COPY ./src src

RUN cargo install --locked --path .

# Final stage

FROM debian:stable-slim
RUN apt-get update && apt-get -y install libssl-dev ca-certificates wget curl git

COPY --from=cargo-build /usr/local/cargo/bin/notion2ics /bin

ENTRYPOINT ["notion2ics"]