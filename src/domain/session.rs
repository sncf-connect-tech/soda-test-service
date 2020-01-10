use std::fmt;

pub enum SessionStatus {
  CREATING,
  COMMAND,
  DELETING,
}

impl fmt::Display for SessionStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      SessionStatus::CREATING => write!(f, "SESSION_CREATING"),
      SessionStatus::COMMAND => write!(f, "SESSION_URL_COMMAND"),
      SessionStatus::DELETING => write!(f, "SESSION_DELETING"),
    }
  }
}
