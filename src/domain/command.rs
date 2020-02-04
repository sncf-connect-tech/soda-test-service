#[derive(Debug, Default, Deserialize)]
pub struct Command {
    pub url: Option<String>,
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
}
