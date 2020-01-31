use bytes::Bytes;
use hyper::{Body, Method, Request, Response};
use std::net::{SocketAddr, ToSocketAddrs};
use url::Url;

use crate::inspector;
use crate::AppState;

#[derive(Debug)]
pub struct RequestToInspect<'m, 'b> {
    pub method: &'m Method,
    pub path: String,
    pub body: &'b Bytes,
}

/// Proxy a Selenium request (from a Selenium client) to the hub.
/// This function also inspect the content in order to write some logs / insights.
pub async fn forward(req: Request<Body>, state: AppState) -> Result<Response<Body>, hyper::Error> {
    println!("la request est : {:?}", req);
    let forward_url = state.forward_uri;
    let out_addr: SocketAddr = forward_url.to_socket_addrs().unwrap().next().unwrap();
    let method = req.method().to_owned();
    let path = &req
        .uri()
        .path_and_query()
        .map(|x| x.to_string())
        .unwrap_or_else(|| "".to_string());

    info!("{} {}", method, path);

    let uri_string = format!("http://{}{}", out_addr, path);
    // todo : handle error
    let url = Url::parse(&uri_string)
        .map_err(|err| error!("err : {}", err))
        .unwrap();

    let body_bytes = hyper::body::to_bytes(req).await?;

    let request_to_inspect = RequestToInspect {
        path: String::from(path),
        method: &method,
        body: &body_bytes,
    };

    inspector::inspect(request_to_inspect).await;

    // todo : handle errors
    // Recreate a request based on the client http request.
    // We use the body, the method and the url provided from the client.
    // Example : POST /session
    // the path is /session, the method is POST and the data is the request body (bytes).
    // Then we send the the request to the hub and we retrieve the response asynchronously.
    let response = state
        .client
        .request(method, url) // POST http://127.0.0.1:4444/session
        .body(body_bytes)
        .send()
        .await
        .map_err(|err| error!("err  for response unwrap : {}", err))
        .unwrap();

    // We retrieve the response body as bytes which is useful if we need
    // to deserialize the data. For example if we need to retrieve the
    // session id once a session is created on the hub.
    let response_body = response
        .bytes()
        .await
        .map_err(|err| error!("err for response body unwrap : {}", err))
        .unwrap();

    // Return the response (from the hub) to the Selenium client.
    Ok(Response::new(Body::from(response_body)))
}
