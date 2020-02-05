use crate::domain;
use crate::reverse_proxy;
use bytes::Bytes;
use hyper::Method;
use std::fmt;

#[derive(Debug, PartialEq)]
struct CreateEvent {
    event: domain::session::SessionStatus,
    desired_capabilities: domain::DesiredCapabilities,
}
#[derive(Debug, PartialEq)]
struct CommandEvent {
    event: domain::session::SessionStatus,
    session_id: String,
    url: String,
}

impl fmt::Display for CommandEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.event, self.session_id, self.url)
    }
}

impl fmt::Display for CreateEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.event, self.desired_capabilities)
    }
}

pub async fn inspect<'m, 'b>(request: reverse_proxy::RequestToInspect<'m, 'b>) {
    let method = request.method.to_owned();
    let path = request.path;

    info!("Inspecting the request {} {}", method, path);

    let body = request.body;

    if method == Method::DELETE {
        info!("{}", capture_delete_event(path).await);
    } else if method == Method::POST && is_a_new_session(&path) {
        info!("{}", capture_create_event(body).await);
    } else if method == "POST" && !is_a_new_session(&path) {
        if let Some(url_event) = capture_url_event(path, body) {
            info!("{}", url_event);
        }
    }
}

async fn capture_delete_event(path: String) -> String {
    let session_id = session_id_of_path(path).unwrap_or_else(|| "".to_string());

    format!(
        "[{}] [{}]",
        domain::session::SessionStatus::Deleting,
        session_id
    )
    // user IP/ID | session status | session ID
}

/// Capture new sessions events
async fn capture_create_event(body: &Bytes) -> CreateEvent {
    let capabilities: domain::Capabilities = serde_json::from_slice(body)
        .map_err(|_| {
            error!(
                "Fail to deserialize the capabilities for the given payload : {:?}",
                body
            );
        })
        .unwrap_or_else(|_| domain::Capabilities::new());

    let desired_capabilities = capabilities.desired_capabilities;

    CreateEvent {
        event: domain::session::SessionStatus::Creating,
        desired_capabilities,
    }
    //status session pass to creating  user IP/ID | session status | Platform | Browser
}

/// Capture asked url events
fn capture_url_event(path: String, body: &Bytes) -> Option<CommandEvent> {
    if path.contains("/url") {
        let command: domain::Command = serde_json::from_slice(body)
            .map_err(|_| {
                error!(
                    "Fail to deserialize the capabilities for the given payload : {:?}",
                    body
                );
            })
            .unwrap_or_else(|_| domain::Command::new());

        let session = session_id_of_path(path).unwrap_or_else(|| "".to_string());

        // user IP/ID | session_status | session ID | url_command | url
        return Some(CommandEvent {
            event: domain::session::SessionStatus::UrlCommand,
            session_id: session,
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
    // we need to verify if the path is to good format
    if !path.contains("wd/hub/session") {
        // warning!("");
        return None;
    }
    //check if the path contains the good string

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_delete_event_should_return_a_well_formated_log_with_the_session_id() {
        let path = "/wd/hub/session/123";
        let delete_event: String = capture_delete_event(path.to_string()).await;

        let expected_delete_event =
            format!("[{}] [{}]", domain::session::SessionStatus::Deleting, "123");

        assert_eq!(
            delete_event, expected_delete_event,
            "The delete event log isn't the expected one."
        );
    }

    #[test]
    fn session_id_of_path_return_none_when_session_id_is_missing() {
        let path: String = "/wd/hub/session".to_string();
        let expected_session = None;
        assert_eq!(session_id_of_path(path), expected_session);
    }

    #[test]
    fn session_id_of_path_return_some_when_session_id_exists() {
        let path: String = "/wd/hub/session/123".to_string();
        assert_eq!(session_id_of_path(path), Some("123".to_string()));
    }

    #[test]
    fn session_id_of_path_return_none_when_path_is_malformed() {
        let path: String = "/bad/hub/session/123".to_string();
        assert_eq!(session_id_of_path(path), None);
    }

    #[test]
    fn session_id_of_path_return_none_when_there_are_a_command() {
        let path: String =
            "/wd/hub/session/5ac4bfb5-0978-4b39-9480-0cf126d2665a/screenshot".to_string();
        assert_eq!(
            session_id_of_path(path),
            Some("5ac4bfb5-0978-4b39-9480-0cf126d2665a".to_string())
        );
    }

    #[tokio::test]
    async fn capture_create_event_return_a_good_format_of_desired_caps() {
        let desired: domain::DesiredCapabilities = domain::DesiredCapabilities {
            browser_name: Some("chrome".to_string()),
            platform: Some("LINUX".to_string()),
            soda_user: Some("Tappas".to_string()),
        };

        let mock_post_http_request_body = r#"
        {
            "capabilities":{
                "desiredCapabilities":{
                    "soda:user":"Tappas",
                    "browserName":"chrome",
                    "testLocal":"false",
                    "acceptSslCerts":true,
                    "platform":"LINUX"
                },
                "requiredCapabilities":{

                }
            },
            "desiredCapabilities":{
                "soda:user":"Tappas",
                "browserName":"chrome",
                "testLocal":"false",
                "acceptSslCerts":true,
                "platform":"LINUX"
            },
            "requiredCapabilities":{

            }
        }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let desired_caps_result = capture_create_event(&body).await;

        let expected_desired_caps = CreateEvent {
            event: domain::SessionStatus::Creating,
            desired_capabilities: desired,
        };

        assert_eq!(desired_caps_result, expected_desired_caps);
    }
    #[tokio::test]
    async fn capture_create_event_return_an_empty_capabilitie_when_fail_to_deserialize_the_capabilities(
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
        let desired_caps_result = capture_create_event(&body).await;

        let desired: domain::DesiredCapabilities = domain::DesiredCapabilities {
            browser_name: Some("".to_string()),
            platform: Some("".to_string()),
            soda_user: Some("".to_string()),
        };

        let expected_desired_caps = CreateEvent {
            event: domain::SessionStatus::Creating,
            desired_capabilities: desired,
        };

        assert_eq!(desired_caps_result, expected_desired_caps);
    }
    #[tokio::test]
    async fn capture_delete_event_should_return_an_empty_session_id_when_there_is_an_unexpected_path(
    ) {
        let path = "/bad/path/session/123";

        let delete_event: String = capture_delete_event(path.to_string()).await;
        // test the format of the delete event

        let expected_delete_event =
            format!("[{}] [{}]", domain::session::SessionStatus::Deleting, "");

        assert_eq!(
            delete_event, expected_delete_event,
            "The delete event shouldn't contains the session id but an empty string."
        );
    }
    #[tokio::test]
    async fn capture_url_event_should_return_session_id_urlcommand_and_session_status() {
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

        assert_eq!(capture_event, expected_event);
    }
    #[tokio::test]
    async fn capture_create_event_should_return_none_when_the_path_does_not_contain_url() {
        let mock_post_http_request_body = r#"
        {
            "url":"https://duckduckgo.com/"
         }"#;

        let body = Bytes::from(mock_post_http_request_body);
        let path = " /wd/hub/session/f52c41e5-3c3f-4cf3-9fe2-963e4a744aa7".to_string();

        let capture_event = capture_url_event(path, &body);

        assert_eq!(capture_event, None);
    }
    #[test]
    fn is_a_new_session_return_splitted_path() {
        let path = "/wd/hub/session".to_string();
        assert_eq!(is_a_new_session(&path), true);
    }
    #[test]
    fn is_a_new_session_return_false_when_given_bad_path() {
        let path = "/wd/hub/session/1234".to_string();
        assert_eq!(is_a_new_session(&path), false);
    }
}
