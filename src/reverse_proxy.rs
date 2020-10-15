use crate::inspector;
use bytes::Bytes;
use hyper::{Body, Method, Request, Response};
use reqwest::Client;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use url::Url;
use uuid::Uuid;

pub struct RequestToInspect<'m, 'b> {
    pub id: Uuid,
    pub method: &'m Method,
    pub path: String,
    pub body: &'b Bytes,
}

/// Proxy a Selenium request (from a Selenium client) to the hub.
/// This function also inspect the content in order to write some logs / insights.
pub async fn forward(
    req: Request<Body>,
    state: SocketAddr,
    timeout: u32,
) -> Result<Response<Body>, hyper::Error> {
    let request_id = Uuid::new_v4();
    let forward_url = state.to_string();
    let out_addr: SocketAddr = forward_url.to_socket_addrs().unwrap().next().unwrap();
    let method = req.method().to_owned();
    let path = &req
        .uri()
        .path_and_query()
        .map(|x| x.to_string())
        .unwrap_or_else(|| "".to_string());

    info!("{:?} {} {}", request_id, method, path);

    let uri_string = format!("http://{}{}", out_addr, path);

    let url = Url::parse(&uri_string)
        .map_err(|err| error!("err : {}", err))
        .unwrap();

    let body_bytes = hyper::body::to_bytes(req).await?;

    let request_to_inspect = RequestToInspect {
        id: request_id,
        path: String::from(path),
        method: &method,
        body: &body_bytes,
    };

    inspector::inspect(request_to_inspect).await;

    let is_a_new_create_session = inspector::is_a_new_create_session(method.to_owned(), &path);

    // If the request to forward is a create session, we remove the timeout be cause the request is not finished
    // while it's in the grid queue
    let client = match is_a_new_create_session {
        true => Client::builder()
            .build()
            .expect("Can't create the http client."),
        false => Client::builder()
            .timeout(Duration::from_secs(timeout.into()))
            .build()
            .expect("Can't create the http client."),
    };

    // Send the request with a retry if the request is not a create session
    // If the last try is an error, the current thread panics
    let response = send_request(
        client.to_owned(),
        method.to_owned(),
        request_id.to_owned(),
        url.to_owned(),
        body_bytes.to_owned(),
        is_a_new_create_session,
    )
    .await
    .unwrap();

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
    request_id: Uuid,
    url: Url,
    body_bytes: Bytes,
    is_a_new_create_session: bool
) -> Result<reqwest::Response, String> {
    let mut tries: usize = 1;
    loop {
        let res = client
            .request(method.to_owned(), url.to_owned())
            .body(body_bytes.to_vec())
            .send()
            .await
            .map_err(|err| {
                format!(
                    "Request Id : {:?} Try number {} in error for response unwrap : {}",
                    request_id, tries, err
                )
            });
        match res {
            Err(e) if tries <= 3 && !is_a_new_create_session => {
                tries += 1;
                log::error!("{}", e);
            }
            res => return res,
        }
    }
}