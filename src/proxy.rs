use crate::inspector;
use bytes::Bytes;
use hyper::{Body, Method, Request, Response};
use reqwest::Client as HttpClient;
use std::net::{SocketAddr, ToSocketAddrs};
use url::Url;

use crate::cli;

#[derive(Debug)]
pub struct RequestToInspect<'m, 'b> {
    pub method: &'m Method,
    pub path: String,
    pub body: &'b Bytes,
}

/// Proxy a Selenium request (from a Selenium client) to the hub.
/// This function also inspect the content in order to write some logs / insights.
pub async fn proxy(req: Request<Body>, client: HttpClient) -> Result<Response<Body>, hyper::Error> {
    let matches = cli::init();
    let forwarded = matches.value_of("forward").unwrap();
    let forward_url = forwarded.to_socket_addrs().unwrap().next().unwrap();
    let out_addr: SocketAddr = forward_url;
    let method = req.method().to_owned();
    let path = &req
        .uri() // http://localhost:8080 -> on laisse
        .path_and_query() // on récupère juste /session?param1...
        .map(|x| x.to_string())
        .unwrap_or_else(|| "".to_string());

    info!("{} {}", method, path);

    let uri_string = format!("http://{}{}", out_addr, path);
    // todo : handle error
    let url = Url::parse(&uri_string)
        .map_err(|err| error!("err : {}", err))
        .unwrap();
    info!("Url : {}", url);

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
    let response = client
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

    //serde the response body to json,(with the html too much verbose)
    // if !response_body.is_empty() {
    //     let json_res: serde_json::Value = serde_json::from_slice(&response_body)
    //         .map_err(|err| error!("err for response body to json : {}", err))
    //         .unwrap();
    //     info!("json response : {}", json_res);
    // }

    // Return the response (from the hub) to the Selenium client.
    Ok(Response::new(Body::from(response_body)))
}
