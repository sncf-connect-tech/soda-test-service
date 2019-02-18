#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use actix_web::{http::Method, middleware::Logger, server, App, HttpResponse};
use clap::value_t;
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
    let listen_addr = matches.value_of("listen_addr").unwrap();
    let listen_port = value_t!(matches, "listen_port", u16).unwrap_or_else(|e| e.exit());
    let forwarded_addr = matches.value_of("forward_addr").unwrap();
    let forwarded_port = value_t!(matches, "forward_port", u16).unwrap_or_else(|e| e.exit());

    // Verify and build the forward URL.
    let forward_url = Url::parse(&format!(
        "http://{}",
        (forwarded_addr, forwarded_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap()
    ))
    .unwrap();

    info!(
        "Server will listen on port {} and forward to {}",
        listen_port, forward_url
    );

    // Run the server with a state containing the forward url and the default credentials.
    // The server spawns a number of workers equals to the number of logical CPU cores,
    // each in its own thread.
    server::new(move || {
        let state =
            domain::AppState::init(forward_url.clone(), auth_user.clone(), auth_pwd.clone());

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
    .bind((listen_addr, listen_port))
    .expect("Cannot bind listening port")
    .system_exit()
    .run();
}
