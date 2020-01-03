#[macro_use]
extern crate serde_derive;

use bytes::Bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Error as HyperError, Method, Request, Response, Server, Uri};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
  pub desired_capabilities: DesiredCapabilities,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredCapabilities {
  browser_name: Option<String>,
  platform: Option<String>,
  // #[serde(rename(deserialize = "soda:user"))]
  soda_user: Option<String>,
}

impl DesiredCapabilities {
  fn new() -> DesiredCapabilities {
    DesiredCapabilities {
      browser_name: None,
      platform: None,
      soda_user: None,
    }
  }
}

impl Capabilities {
  pub fn new() -> Capabilities {
    Capabilities {
      desired_capabilities: DesiredCapabilities::new(),
    }
  }
}

#[derive(Debug)]
struct RequestToInspect<'m, 'b> {
  method: &'m Method,
  path: String,
  body: &'b Bytes,
}

enum SessionStatus {
  CREATING,
  CREATED,
  DELETING,
  DELETED,
}

struct Session {
  id: Option<String>,
  status: SessionStatus,
  user: Option<String>,
}

#[tokio::main]
async fn main() {
  let in_addr = ([127, 0, 0, 1], 8080).into();

  let make_svc = make_service_fn(|_conn| {
    async {
      // This is the `Service` that will handle the connection.
      // `service_fn` is a helper to convert a function that
      // returns a Response into a `Service`.
      Ok::<_, HyperError>(service_fn(proxy))
    }
  });

  let server = Server::bind(&in_addr).serve(make_svc);

  println!("Listening on http://{}", in_addr);

  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }
}

async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
  let out_addr: SocketAddr = ([127, 0, 0, 1], 4444).into();
  let method = req.method().to_owned();
  let path = &req
    .uri()
    .path_and_query()
    .map(|x| x.to_string())
    .unwrap_or_else(|| "".to_string());

  println!("{} {}", method, path);

  let uri_string = format!("http://{}{}", out_addr, path);
  // todo : handle error
  let url = Url::parse(&uri_string)
    .map_err(|err| eprintln!("err : {}", err))
    .unwrap();
  println!("Url : {}", url);

  let client = HttpClient::new();
  let body_bytes = hyper::body::to_bytes(req).await?;

  let request_to_inspect = RequestToInspect {
    path: path.to_string(),
    method: &method,
    body: &body_bytes,
  };

  inspect(request_to_inspect).await;

  // todo : handle errors
  let response = client
    .request(method, url)
    .body(body_bytes)
    .send()
    .await
    .map_err(|err| eprintln!("err for response unwrap : {}", err))
    .unwrap();

  let response_body = response
    .bytes()
    .await
    .map_err(|err| eprintln!("err for response body unwrap : {}", err))
    .unwrap();

  // if !response_body.is_empty() {
  //   let json_res: serde_json::Value = serde_json::from_slice(&response_body)
  //     .map_err(|err| eprintln!("err for response body to json : {}", err))
  //     .unwrap();
  //   println!("json response : {}", json_res);
  // }

  Ok(Response::new(Body::from(response_body)))
}

async fn inspect<'m, 'b>(request: RequestToInspect<'m, 'b>) {
  let method = request.method.to_owned();
  let path = request.path;

  println!("Inspecting the request {} {}", method, path);

  let body = request.body;

  if method == Method::DELETE {
    capture_delete_event(path).await;
  } else if method == Method::POST && is_a_new_session(path) {
    capture_create_event(body).await;
  }
}

async fn capture_delete_event(path: String) {
  let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

  // user IP/ID | session status | session ID
  println!("[DELETING_WAIT_FOR_FORMATTER] [{}]", session_id);
}

/// Capture new sessions and log
async fn capture_create_event(body: &Bytes) {
  let capabilities: Capabilities = serde_json::from_slice(body)
    .map_err(|_| {
      eprintln!(
        "Fail to deserialize the capabilities for the given payload : {:?}",
        body
      );
    })
    .unwrap_or_else(|_| Capabilities::new());

  let desired_caps = capabilities.desired_capabilities;

  // user IP/ID | session status | Platform | Browser
  println!("[CREATING_WAIT_FOR_FORMATTER] {:?}", desired_caps);
}

/// Split the path to determine if it's a new session
/// (the path doesn't contain the session's id) or if it's
/// an existing session (the path contains the session's id).
/// If the head and the tail of the path are empty,
/// it's a new session event that we want to capture.
fn is_a_new_session(path: String) -> bool {
  let splitted_path: Vec<&str> = path
    .split("/wd/hub/session")
    .filter(|item| !item.is_empty())
    .collect();

  splitted_path.is_empty()
}

/// Split the given path and try to retrieve the
/// session's id.
fn session_id_of_path(path: String) -> Option<String> {
  // Try to get the session's id part
  // e.g. possible patterns :
  // /wd/hub/session
  // /wd/hub/session/:id
  // /wd/hub/session/:id/:cmd
  let tail: Vec<&str> = path
    .split("/wd/hub/session")
    .filter(|item| !item.is_empty())
    .collect();

  // Check if there is a remainder with a session's id
  // e.g. possible patterns :
  // /:id
  // /:id/:cmd
  if !tail.is_empty() {
    let remainder: Vec<&str> = tail[0].split('/').filter(|s| !s.is_empty()).collect();

    return Some(remainder[0].to_string());
  }

  None
}
