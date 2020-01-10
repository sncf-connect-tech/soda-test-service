use bytes::Bytes;
use hyper::Method;

use crate::domain;
use crate::proxy;

pub async fn inspect<'m, 'b>(request: proxy::RequestToInspect<'m, 'b>) {
    let method = request.method.to_owned();
    let path = request.path;

    info!("Inspecting the request {} {}", method, path);

    let body = request.body;

    if method == Method::DELETE {
        info!("{}", domain::session::SessionStatus::DELETING);
        capture_delete_event(path).await;
    } else if method == Method::POST && is_a_new_session(&path) {
        info!("{}", domain::session::SessionStatus::CREATING);
        capture_create_event(body).await;
    } else if method == "POST" && !is_a_new_session(&path) {
        capture_url_event(path, body);
    }
}

async fn capture_delete_event(path: String) {
    let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

    // user IP/ID | session status | session ID
    info!(
        "[{}] [{}]",
        domain::session::SessionStatus::DELETED,
        session_id
    );
}

/// Capture new sessions events
async fn capture_create_event(body: &Bytes) {
    let capabilities: domain::Capabilities = serde_json::from_slice(body)
        .map_err(|_| {
            error!(
                "Fail to deserialize the capabilities for the given payload : {:?}",
                body
            );
        })
        .unwrap_or_else(|_| domain::Capabilities::new());

    let desired_caps = capabilities.desired_capabilities;

    //status session pass to creating  user IP/ID | session status | Platform | Browser

    info!(
        "[{}] {:?}",
        domain::session::SessionStatus::CREATED,
        desired_caps,
    );
}

/// Capture asked url events
fn capture_url_event(path: String, body: &Bytes) {
    if path.contains("/url") {
        let command: domain::Command = serde_json::from_slice(body)
            .map_err(|_| {
                error!(
                    "Fail to deserialize the capabilities for the given payload : {:?}",
                    body
                );
            })
            .unwrap_or_else(|_| domain::Command::new());

        let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

        // user IP/ID | session_status | session ID | url_command | url
        info!(
            "[{}] [{}] [{}] [{}]",
            domain::session::SessionStatus::COMMAND,
            session_id,
            "url",
            command.url()
        );
    }
}

/// Split the path to determine if it's a new session
/// (the path doesn't contain the session's id) or if it's
/// an existing session (the path contains the session's id).
/// If the head and the tail of the path are empty,
/// it's a new session event that we want to capture.
fn is_a_new_session(path: &str) -> bool {
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
