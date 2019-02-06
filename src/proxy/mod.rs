use actix_web::{
    client::ClientRequestBuilder, client::ClientRequest, client::ClientResponse,
    http, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse, http::Method
};
use futures::{future, Future, Stream};
use std::time::Duration;
use domain::AppState;
use bytes::Bytes;
use session;

pub fn forward(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let mut new_url = req.state().forward_url.clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let mut client_builder = ClientRequest::build_from(req);
    client_builder
        .no_default_headers()
        .uri(new_url)
        .timeout(Duration::from_secs(60)); // <- the max timeout we allow for Selenium commands

    // inspect the http request and then process
    let mut client_request = inspect_and_stream(&req, &mut client_builder).unwrap();

    if let Some(addr) = req.peer_addr() {
        match client_request.headers_mut().entry("x-forwarded-for") {
            Ok(http::header::Entry::Vacant(entry)) => {
                let addr = format!("{}", addr.ip());
                entry.insert(addr.parse().unwrap());
            }
            Ok(http::header::Entry::Occupied(mut entry)) => {
                let addr = format!("{}, {}", entry.get().to_str().unwrap(), addr.ip());
                entry.insert(addr.parse().unwrap());
            }
            _ => unreachable!(),
        }
    }

    // TODO : Handle specific errors
    client_request
        .send()
        .map_err(|e| {
            println!("error = {:?}", e);
            Error::from(e)
        })
        .and_then(construct_response)
        .responder()
}

fn construct_response(
    resp: ClientResponse,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let mut client_resp = HttpResponse::build(resp.status());
    for (header_name, header_value) in
        resp.headers().iter().filter(|(h, _)| *h != "connection")
    {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    // Stream the response by returning chunks.
    Box::new(future::ok(client_resp.streaming(resp.payload())))
}

/// Inspect the given http request and then process to the stream.
/// When the body isn't empty, chunks are inspected to retrieve
/// information about the current test session.
fn inspect_and_stream(
    req: &HttpRequest<AppState>,
    client_builder: &mut ClientRequestBuilder
) -> Result<ClientRequest, Error> {
    let req2 = req.clone(); // we need to extend the lifetime of req

    match req.method() {
        &Method::GET => client_builder.finish(), // <- Get requests have an empty body
        &Method::DELETE => { // <- Delete requests have an empty body
            session::inspect(&req2, Bytes::from(&b""[..]));

            client_builder.finish()
        },
        _ => { // In all the other cases we inspect and stream the body
            client_builder.streaming(req.payload()
                .map(move |chunk| session::inspect(&req2, chunk))
            )
        }
    }
}
