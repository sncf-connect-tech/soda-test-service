#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use env_logger;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Error as HyperError, Server};
use std::net::{SocketAddr, ToSocketAddrs};
use url::Url;
mod cli;
mod domain;
mod inspector;
mod proxy;

#[tokio::main]
async fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();
  let matches = cli::init();

  // Configure addresses to listen and forward.
  let listen = matches.value_of("listen").unwrap();
  let forwarded = matches.value_of("forward").unwrap();

  // Used to give a more verbose output. (all info logs)
  let verbose = matches.occurrences_of("verbose");

  // Configure the timeout for the proxy, default to 60s
  let timeout = value_t!(matches, "timeout", u32).unwrap_or(60);

  // Verify and build the forward URL.
  let forward_url = Url::parse(&format!(
    "http://{}",
    forwarded.to_socket_addrs().unwrap().next().unwrap()
  ))
  .unwrap();

  let in_addr = ([127, 0, 0, 1], 8080).into();

  let make_svc = make_service_fn(|_conn| {
    async {
      // This is the `Service` that will handle the connection.
      // `service_fn` is a helper to convert a function that
      // returns a Response into a `Service`.
      Ok::<_, HyperError>(service_fn(proxy::proxy))
    }
  });

  let server = Server::bind(&in_addr).serve(make_svc);

  info!("Listening on http://{}", in_addr);

  if let Err(e) = server.await {
    error!("server error: {}", e);
  }
}
