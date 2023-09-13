FROM rust:latest as build
WORKDIR /app
COPY . .
RUN cargo build --release --example=io_bencher

FROM ubuntu:22.04
COPY --from=build /app/target/release/examples/io_bencher /usr/local/bin/
