FROM rust:1.31

# Add the released binary
ADD target/release/soda_test_service /soda/soda_test_service
ADD .env /soda/.env
RUN ls /soda/*

# Install dependencies
RUN apt-get -qqy update \
  && apt-get -qqy install \
  openssl \
  libssl-dev \
  pkg-config \
  && rm -rf /var/lib/apt/lists/* /var/cache/apt/*

WORKDIR /soda
EXPOSE 8080

# This command run the test service which listen on the specified address:port
# and forward http requests to the specified address:port.
# Arguments are : LISTEN ADDR, LISTEN PORT, FWD ADDR, FWD PORT
CMD /soda/soda_test_service 0.0.0.0 8080 $HUB_PORT_4444_TCP_ADDR $HUB_PORT_4444_TCP_PORT
