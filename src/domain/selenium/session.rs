use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum SessionStatus {
  Creating,
  Deleting,
  RunCommand,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
  session_id: String,
}

impl fmt::Display for SessionStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      SessionStatus::Creating => write!(f, "SESSION_CREATING"),
      SessionStatus::Deleting => write!(f, "SESSION_DELETING"),
      SessionStatus::RunCommand => write!(f, "SESSION_RUN_COMMAND"),
    }
  }
}
