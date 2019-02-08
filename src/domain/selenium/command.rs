use serde_json::{self, Result};

#[derive(Debug, Default, Deserialize)]
pub struct Command {
  url: Option<String>,
}

impl Command {
  pub fn new() -> Command {
    Command {
      url: Some("".to_string()),
    }
  }

  pub fn url(self) -> String {
    self.url.unwrap_or_else(|| "".to_string())
  }

  pub fn deserialize(content: &str) -> Result<Command> {
    serde_json::from_str(content)
  }
}
