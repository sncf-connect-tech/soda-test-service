#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use actix_web::{http::Method, middleware::Logger, server, App, HttpResponse};
use env_logger;
use std::env;
use std::net::ToSocketAddrs;
use url::Url;

mod cli;
mod domain;
mod middlewares;
mod proxy;
mod session;

fn main() {
  std::env::set_var("RUST_LOG", "info");
  env_logger::init();

  // The environment variables used for the Basic-Auth.
  // In the future it will be replaced by a database for the hot reload.
  let auth_user = env::var("AUTH_USER").unwrap_or_else(|_| "".to_string());
  let auth_pwd = env::var("AUTH_PWD").unwrap_or_else(|_| "".to_string());

  // Start the parsing of arguments.
  let matches = cli::init();

  // Configure addresses to listen and forward.
  let listen = matches.value_of("listen").unwrap();
  let forwarded = matches.value_of("forward").unwrap();

  // Verify and build the forward URL.
  let forward_url = Url::parse(&format!(
    "http://{}",
    forwarded.to_socket_addrs().unwrap().next().unwrap()
  ))
  .unwrap();

  info!(
    "Server will listen on {} and forward to {}",
    listen, forward_url
  );

  // Run the server with a state containing the forward url and the default credentials.
  // The server spawns a number of workers equals to the number of logical CPU cores,
  // each in its own thread.
  server::new(move || {
    let state = domain::AppState::init(forward_url.clone(), auth_user.clone(), auth_pwd.clone());

    App::with_state(state)
      .middleware(Logger::default())
      .resource("/healthcheck", |r| {
        r.method(Method::GET).f(|_| HttpResponse::Ok())
      })
      .default_resource(|r| {
        r.middleware(middlewares::Auth);
        r.f(proxy::forward)
      })
  })
  .bind(listen)
  .expect("Cannot bind listening port")
  .system_exit()
  .run();
}
