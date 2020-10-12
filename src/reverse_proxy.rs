use std::net::{SocketAddr, ToSocketAddrs};

use bytes::Bytes;
use hyper::{Body, Method, Request, Response};
use reqwest::Client;
use url::Url;

use crate::inspector;
use crate::AppState;

pub struct RequestToInspect<'m, 'b> {
    pub method: &'m Method,
    pub path: String,
    pub body: &'b Bytes,
}

/// Proxy a Selenium request (from a Selenium client) to the hub.
/// This function also inspect the content in order to write some logs / insights.
pub async fn forward(req: Request<Body>, state: AppState) -> Result<Response<Body>, hyper::Error> {
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

    // Custom dirty retry because it's impossible to make a retry with an async function inside, sorry
    // TODO make it better
    let response = match send_request(
        state.client.to_owned(),
        method.to_owned(),
        url.to_owned(),
        body_bytes.to_owned(),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            match send_request(
                state.client.to_owned(),
                method.to_owned(),
                url.to_owned(),
                body_bytes.to_owned(),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => send_request(
                    state.client.to_owned(),
                    method.to_owned(),
                    url.to_owned(),
                    body_bytes.to_owned(),
                )
                .await
                .unwrap(),
            }
        }
    };

    // Rebuild the response by adding the parsed body
    let mut response_builder = hyper::Response::builder().status(response.status());

    // We copy the headers from the hub response to the client response.
    let headers = response_builder.headers_mut().unwrap();

    for (key, value) in response.headers() {
        headers.insert(key, value.to_owned());
    }

    // We retrieve the response body as bytes which is useful if we need
    // to deserialize the data. For example if we need to retrieve the
    // session id once a session is created on the hub.
    let response_body = response
        .bytes()
        .await
        .map_err(|err| error!("err for response body unwrap : {}", err))
        .unwrap();

    // Return the response (from the hub) to the Selenium client.
    Ok(response_builder.body(Body::from(response_body)).unwrap())
}

// Recreate a request based on the client http request.
// We use the body, the method and the url provided from the client.
// Example : POST /session
// the path is /session, the method is POST and the data is the request body (bytes).
// Then we send the the request to the hub and we retrieve the response asynchronously.
pub async fn send_request(
    client: Client,
    method: Method,
    url: Url,
    body_bytes: Bytes,
) -> Result<reqwest::Response, ()> {
    client
        .request(method.to_owned(), url.to_owned())
        .body(body_bytes)
        .send()
        .await
        .map_err(|err| error!("First try in error for response unwrap : {}", err))
}
