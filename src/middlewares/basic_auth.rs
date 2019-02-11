use crate::domain::AppState;
use actix_web::middleware::{Middleware, Started};
use actix_web::{FromRequest, HttpRequest, Result};
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

pub struct Auth;

impl Middleware<AppState> for Auth {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
        // The credentials are verified and configured in the main function.
        let auth_user = &req.state().auth_user;
        let auth_pwd = &req.state().auth_pwd;

        // The realm configuration.
        let mut config = Config::default();
        config.realm("SODA Test Service");

        // Check auth information.
        let auth = BasicAuth::from_request(&req, &config)?;

        if auth.username() == auth_user && auth.password() == Some(&auth_pwd) {
            Ok(Started::Done)
        } else {
            Err(AuthenticationError::from(config).into())
        }
    }
}
