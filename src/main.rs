use hyper::client::ResponseFuture;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Error as HyperError, Method, Request, Response, Server, Uri};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde_json;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use url::Url;

#[derive(Debug)]
pub struct Capabilities {
  pub desired_capabilities: DesiredCapabilities,
}

#[derive(Debug)]
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
struct RequestToInspect<'b> {
  method: Method,
  path: String,
  body: &'b Body,
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

fn capture_new_session(session: Session) -> Session {
  unimplemented!();
}

fn capture_event(session: Session) -> Session {
  unimplemented!();
}

fn forward(req: Request<Body>) -> Response<Body> {
  unimplemented!();
}

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

  let buf = hyper::body::to_bytes(req).await?;
  // todo : handle error
  if !buf.is_empty() {
    let json_req: serde_json::Value = serde_json::from_slice(&buf)
      .map_err(|err| eprintln!("err : {}", err))
      .unwrap();
    println!("json req : {}", json_req);
  }

  println!("Buffer : {:?}", buf);

  // todo : handle errors
  let response = client
    .request(method, url)
    .body(buf)
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
