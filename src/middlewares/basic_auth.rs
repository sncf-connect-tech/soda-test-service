use crate::domain::AppState;
use actix_web::middleware::{Middleware, Started};
use actix_web::{FromRequest, HttpRequest, Result};
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

pub struct Auth;

impl Middleware<AppState> for Auth {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
        // The credentials are configured in the main.
        let auth_user = &req.state().auth_user;
        let auth_pwd = &req.state().auth_pwd;

        // If the username and the password are empty, we consider that the test service has
        // been configured to run without the Basic-Auth.
        if auth_user.is_empty() && auth_pwd.is_empty() {
            return Ok(Started::Done);
        }

        // The realm configuration.
        let mut config = Config::default();
        config.realm("SODA Test Service");

        let auth = BasicAuth::from_request(&req, &config)?;

        // Check auth information.
        if auth.username() == auth_user && auth.password() == Some(&auth_pwd) {
            Ok(Started::Done)
        } else {
            Err(AuthenticationError::from(config).into())
        }
    }
}
