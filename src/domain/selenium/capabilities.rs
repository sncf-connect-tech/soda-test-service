use serde_json::{self, Result};
use std::fmt;


#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
  pub desired_capabilities: DesiredCapabilities,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DesiredCapabilities {
  browser_name: Option<String>,
  platform: Option<String>,
  user: Option<String>,
}

impl Capabilities {
  pub fn new() -> Capabilities {
    Capabilities {
      desired_capabilities: DesiredCapabilities::new(),
    }
  }

  pub fn deserialize(json: &str) -> Result<Capabilities> {
    serde_json::from_str(json)
  }
}

impl DesiredCapabilities {
  fn new() -> DesiredCapabilities {
    DesiredCapabilities {
      browser_name: Some("".to_string()),
      platform: Some("".to_string()),
      user: Some("".to_string()),
    }
  }

  pub fn get_platform(&self) -> String {
    match &self.platform {
      Some(platform) => platform.to_string(),
      None => "".to_string(),
    }
  }

  pub fn get_browser_name(&self) -> String {
    match &self.browser_name {
      Some(browser_name) => browser_name.to_string(),
      None => "".to_string(),
    }
  }

  pub fn get_user(&self) -> String {
    match &self.user {
      Some(user) => user.to_string(),
      None => "".to_string(),
    }
  }
}

impl fmt::Display for DesiredCapabilities {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "browser:{}, platform:{}, user:{}",
      self
        .browser_name
        .to_owned()
        .unwrap_or_else(|| "".to_string()),
      self.platform.to_owned().unwrap_or_else(|| "".to_string()),
      self.user.to_owned().unwrap_or_else(|| "".to_string())
    )
  }
}
