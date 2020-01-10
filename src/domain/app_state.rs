use reqwest::Client as HttpClient;

pub struct AppState {
    pub client: HttpClient,
    pub forward: String,
}
