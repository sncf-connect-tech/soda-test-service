use std::fmt;

pub enum SessionStatus {
  CREATING,
  CREATED,
  COMMAND,
  DELETING,
  DELETED,
}

impl fmt::Display for SessionStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      SessionStatus::CREATING => write!(f, "SESSION_CREATING"),
      SessionStatus::CREATED => write!(f, "SESSION_CREATED"),
      SessionStatus::COMMAND => write!(f, "SESSION_COMMAND"),
      SessionStatus::DELETING => write!(f, "SESSION_DELETING"),
      SessionStatus::DELETED => write!(f, "SESSION_DELETED"),
    }
  }
}
