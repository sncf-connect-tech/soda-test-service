use url::Url;

pub struct AppState {
    pub forward_url: Url,
    pub auth_user: String,
    pub auth_pwd: String,
    pub timeout: u32,
}

impl AppState {
    pub fn init(forward_url: Url, auth_user: String, auth_pwd: String, timeout: u32) -> AppState {
        AppState {
            forward_url,
            auth_user,
            auth_pwd,
            timeout,
        }
    }
}
