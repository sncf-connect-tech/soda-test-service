#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
  pub desired_capabilities: DesiredCapabilities,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredCapabilities {
  browser_name: Option<String>,
  platform: Option<String>,
  #[serde(rename(deserialize = "soda:user"))]
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
