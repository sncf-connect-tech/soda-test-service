use crate::domain;
use crate::reverse_proxy;
use bytes::Bytes;
use hyper::Method;
use std::fmt;

#[derive(PartialEq)]
struct CreateEvent {
    event: domain::session::SessionStatus,
    desired_capabilities: domain::DesiredCapabilities,
}

#[derive(PartialEq)]
struct CommandEvent {
    event: domain::session::SessionStatus,
    session_id: String,
    url: String,
}

#[derive(PartialEq)]
struct DeleteEvent {
    event: domain::session::SessionStatus,
    session_id: String,
}

impl fmt::Display for CommandEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] [{}] [{}]", self.event, self.session_id, self.url)
    }
}

impl fmt::Display for CreateEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.event, self.desired_capabilities)
    }
}

impl fmt::Display for DeleteEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] [{}]", self.event, self.session_id)
    }
}

pub async fn inspect<'m, 'b>(request: reverse_proxy::RequestToInspect<'m, 'b>) {
    let id = request.id;
    let method = request.method.to_owned();
    let path = request.path;

    let body = request.body;

    if method == Method::DELETE {
        info!("Request Id : {:?}, {}", id, capture_delete_event(path).await);
    } else if is_a_new_create_session(method.to_owned(), &path) {
        info!("Request Id : {:?}, {}", id, capture_create_event(body).await);
    } else if method == "POST" && !is_a_new_session(&path) {
        if let Some(url_event) = capture_url_event(path, body) {
            info!("Request Id : {:?}, {}", id, url_event);
        }
    }
}

pub fn is_a_new_create_session(method: Method, path: &str) -> bool {
  method == Method::POST && is_a_new_session(&path)
}

async fn capture_delete_event(path: String) -> DeleteEvent {
    let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

    DeleteEvent {
        event: domain::session::SessionStatus::Deleting,
        session_id,
    }
}

/// Capture new sessions events
async fn capture_create_event(body: &Bytes) -> CreateEvent {
    let capabilities: domain::Capabilities = serde_json::from_slice(body)
        .map_err(|_| {
            error!(
                "Fail to deserialize the capabilities for the given payload : {}",
                std::str::from_utf8(body).unwrap_or("cannot read the body")
            );
        })
        .unwrap_or_else(|_| domain::Capabilities::new());

    let desired_capabilities = capabilities.desired_capabilities;

    CreateEvent {
        event: domain::session::SessionStatus::Creating,
        desired_capabilities,
    }
}

/// Capture asked url events
fn capture_url_event(path: String, body: &Bytes) -> Option<CommandEvent> {
    if path.contains("/url") {
        let command: domain::Command = serde_json::from_slice(body)
            .map_err(|_| {
                error!(
                    "Fail to deserialize the capabilities for the given payload : {}",
                    std::str::from_utf8(body).unwrap_or("cannot read the body")
                );
            })
            .unwrap_or_else(|_| domain::Command::new());

        let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

        // event | session_status | session ID | url_command | url
        return Some(CommandEvent {
            event: domain::session::SessionStatus::UrlCommand,
            session_id,
            url: command.url(),
        });
    }

    None
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
/// session id.
fn session_id_of_path(path: String) -> Option<String> {
    // we only do the check if the path concerns a valid session endpoint
    if !path.contains("wd/hub/session") {
        return None;
    }
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
        if !remainder.is_empty() {
            return Some(remainder[0].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_delete_event_should_return_a_well_formatted_log_with_the_session_id() {
        let path = "/wd/hub/session/123";
        let delete_event = capture_delete_event(path.to_string()).await;

        let expected_delete_event = DeleteEvent {
            event: domain::session::SessionStatus::Deleting,
            session_id: "123".to_string(),
        };

        assert!(delete_event == expected_delete_event);
    }

    #[test]
    fn session_id_of_path_returns_none_when_session_id_is_missing() {
        let path: String = "/wd/hub/session//".to_string();
        assert!(session_id_of_path(path).is_none());
    }

    #[test]
    fn session_id_of_path_returns_some_when_session_id_exists() {
        let path: String = "/wd/hub/session/123/screenshot".to_string();
        assert_eq!(session_id_of_path(path), Some("123".to_string()));
    }

    #[test]
    fn session_id_of_path_returns_none_when_path_is_malformed() {
        let path: String = "/bad/hub/session/123/screenshot".to_string();
        assert!(session_id_of_path(path).is_none());
    }

    #[tokio::test]
    async fn capture_create_event_returns_a_struct_with_non_empty_desired_capabilities_when_the_http_request_is_valid(
    ) {
        let desired: domain::DesiredCapabilities = domain::DesiredCapabilities {
            browser_name: Some("chrome".to_string()),
            platform: Some("LINUX".to_string()),
            soda_user: Some("user123".to_string()),
        };

        let mock_post_http_request_body = r#"
        {
            "capabilities":{
                "desiredCapabilities":{
                    "soda:user":"user123",
                    "browserName":"chrome",
                    "testLocal":"false",
                    "acceptSslCerts":true,
                    "platform":"LINUX"
                },
                "requiredCapabilities":{

                }
            },
            "desiredCapabilities":{
                "soda:user":"user123",
                "browserName":"chrome",
                "testLocal":"false",
                "acceptSslCerts":true,
                "platform":"LINUX"
            },
            "requiredCapabilities":{

            }
        }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let create_event = capture_create_event(&body).await;

        let expected_create_event = CreateEvent {
            event: domain::SessionStatus::Creating,
            desired_capabilities: desired,
        };

        assert!(create_event == expected_create_event);
    }

    #[tokio::test]
    async fn capture_create_event_returns_a_struct_with_empty_capabilities_when_it_fails_to_deserialize_the_http_request(
    ) {
        let mock_post_http_request_body = r#"
        {
            "capabilities":{
                "desiredCapabilities":{
                    "soda:user":"",
                    "browserName":"",
                    "testLocal":"false",
                    "acceptSslCerts":true,
                    "platform":""
                },
                "requiredCapabilities":{

                }
            },
            "desiredCapabilities":{
                "soda:user":"",
                "browserName":"",
                "testLocal":"false",
                "acceptSslCerts":true,
                "platform":""
            },
            "requiredCapabilities":{

            }
        }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let create_event = capture_create_event(&body).await;

        let desired: domain::DesiredCapabilities = domain::DesiredCapabilities {
            browser_name: Some("".to_string()),
            platform: Some("".to_string()),
            soda_user: Some("".to_string()),
        };

        let expected_create_event = CreateEvent {
            event: domain::SessionStatus::Creating,
            desired_capabilities: desired,
        };

        assert!(create_event == expected_create_event);
    }

    #[tokio::test]
    async fn capture_delete_event_should_return_an_empty_session_id_when_there_is_an_unexpected_path(
    ) {
        let path = "/bad/path/session/123";

        let delete_event = capture_delete_event(path.to_string()).await;

        let expected_delete_event = DeleteEvent {
            event: domain::session::SessionStatus::Deleting,
            session_id: "".to_string(),
        };

        assert!(delete_event == expected_delete_event);
    }

    #[tokio::test]
    async fn capture_url_event_should_return_a_filled_command_event() {
        let mock_post_http_request_body = r#"
        {
            "url":"https://duckduckgo.com/"
         }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let path = "/wd/hub/session/f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7/url".to_string();

        let expected_event = Some(CommandEvent {
            event: domain::session::SessionStatus::UrlCommand,
            session_id: "f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7".to_string(),
            url: "https://duckduckgo.com/".to_string(),
        });

        let capture_event = capture_url_event(path, &body);

        assert!(capture_event == expected_event);
    }

    #[tokio::test]
    async fn capture_url_event_should_return_none_when_the_path_does_not_contain_url() {
        let mock_post_http_request_body = r#"
        {
            "url":"https://duckduckgo.com/"
         }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let path = "/wd/hub/session/f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7".to_string();

        let capture_event = capture_url_event(path, &body);

        assert!(capture_event.is_none());
    }

    #[tokio::test]
    async fn capture_url_event_should_return_an_empty_url_when_the_http_request_is_malformed() {
        let mock_post_http_request_body = r#"
        {
         }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let path = "/wd/hub/session/f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7/url".to_string();

        let expected_event = Some(CommandEvent {
            event: domain::session::SessionStatus::UrlCommand,
            session_id: "f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7".to_string(),
            url: "".to_string(),
        });

        let capture_event = capture_url_event(path, &body);

        assert!(capture_event == expected_event);
    }

    #[test]
    fn is_a_new_session_returns_true_when_the_path_does_not_contain_session_id() {
        let path = "/wd/hub/session".to_string();

        assert!(is_a_new_session(&path));
    }

    #[test]
    fn is_a_new_session_returns_false_when_there_is_a_session_id() {
        let path = "/wd/hub/session/123/screenshot".to_string();

        assert!(!is_a_new_session(&path));
    }

    #[test]
    fn is_a_new_session_returns_false_when_the_path_is_malformed() {
        let path = "/wd/hub/session//screenshot".to_string();

        assert!(!is_a_new_session(&path));
    }
}
