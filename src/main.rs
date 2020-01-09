#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
use env_logger;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response};
use hyper::{Error, Server};
use reqwest::Client as HttpClient;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;
use url::Url;
mod cli;
mod domain;
mod inspector;
mod proxy;

pub struct Client<'t> {
  pub timeout: &'t u32,
}

#[tokio::main]
async fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();
  let matches = cli::init();

  // Configure addresses to listen and forward.
  let listen = matches.value_of("listen").unwrap();
  let forwarded = matches.value_of("forward").unwrap();
  let in_addr = listen.to_socket_addrs().unwrap().next().unwrap();

  // Used to give a more verbose output. (all info logs)
  let verbose = matches.occurrences_of("verbose");

  // Configure the timeout for the proxy, default to 60s
  let timeout = value_t!(matches, "timeout", u32).unwrap_or(60);

  let make_svc = make_service_fn(move |_| {
    async move {
      Ok::<_, Error>(service_fn(move |req: Request<Body>| {
        async move {
          // Build a http client with reqwest
          let client = HttpClient::builder()
            .timeout(Duration::from_secs(timeout.into()))
            .build()
            .expect("Can't create the http client.");

          proxy::proxy(req, client).await
        }
      }))
    }
  });

  let server = Server::bind(&in_addr).serve(make_svc);

  info!(
    "Server will listen on {} and forward to {}",
    listen, forwarded
  );

  if let Err(e) = server.await {
    error!("server error: {}", e);
  }
}
