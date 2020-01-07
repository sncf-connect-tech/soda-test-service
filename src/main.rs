#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use env_logger;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Error as HyperError, Server};

mod domain;
mod inspector;
mod proxy;

#[tokio::main]
async fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();
  let in_addr = ([127, 0, 0, 1], 8081).into();

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
