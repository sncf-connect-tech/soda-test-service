use std::fmt;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub desired_capabilities: DesiredCapabilities,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DesiredCapabilities {
    pub browser_name: Option<String>,
    pub platform: Option<String>,
    #[serde(rename(deserialize = "soda:user"))]
    pub soda_user: Option<String>,
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

impl fmt::Display for DesiredCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(
            f,
            "(browser: {}, platform: {}, user: {})",
            self.browser_name.clone().unwrap_or_else(|| "".to_string()),
            self.platform.clone().unwrap_or_else(|| "".to_string()),
            self.soda_user.clone().unwrap_or_else(|| "".to_string())
        )
    }
}

impl Capabilities {
    pub fn new() -> Capabilities {
        Capabilities {
            desired_capabilities: DesiredCapabilities::new(),
        }
    }
}
