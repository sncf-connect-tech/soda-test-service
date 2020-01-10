use std::fmt;

pub enum SessionStatus {
    Creating,
    UrlCommand,
    Deleting,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SessionStatus::Creating => write!(f, "SESSION_CREATING"),
            SessionStatus::UrlCommand => write!(f, "SESSION_URL_COMMAND"),
            SessionStatus::Deleting => write!(f, "SESSION_DELETING"),
        }
    }
}
