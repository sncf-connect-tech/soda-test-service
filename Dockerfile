# Inspired by https://whitfin.io/speeding-up-rust-docker-builds/
# Step 1 : build the optimized binary
FROM rust:1.46.0 as build

# create a new empty shell project
RUN USER=root cargo new --bin soda-test-service
WORKDIR /soda-test-service

# copy over the manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# this build step will cache the dependencies
# after the first build
RUN cargo build --release
RUN rm src/*.rs

# copy the source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/soda_test_service*
RUN cargo build --release

# Step 2 :# run the application in a slim container
# without cargo, the toolchain, ...
FROM ubuntu:latest

COPY --from=build /soda-test-service/target/release/soda-test-service .
EXPOSE 8080

RUN apt-get update
ENV TZ=Europe/Paris
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get install -y pkg-config libssl-dev

ENV LD_LIBRARY_PATH=/usr/local/lib

# This command run the test service which listen on the specified address:port
# and forward http requests to the specified address:port.
# Arguments are : LISTEN ADDR, LISTEN PORT, FWD ADDR, FWD PORT
CMD ./soda-test-service --listen=0.0.0.0:8080 --forward=$HUB_PORT_4444_TCP_ADDR:$HUB_PORT_4444_TCP_PORT --timeout=$HUB_CLIENT_TIMEOUT
