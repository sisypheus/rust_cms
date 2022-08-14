use std::pin::Pin;

use actix_session::SessionExt;
use actix_web::error::ErrorUnauthorized;
use actix_web::Result;
use actix_web::{Error as AWError, FromRequest, HttpRequest};
use futures_util::Future;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}

impl FromRequest for User {
    type Error = AWError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();

        let session: Result<Option<User>, serde_json::Error> =
            req.get_session().get::<User>("user");

        let user: Option<User> = match session {
            Ok(user) => user,
            Err(_) => None,
        };

        match user {
            Some(user) => Box::pin(async move { Ok(user) }),
            None => Box::pin(async move { Err(ErrorUnauthorized("Unauthorized")) }),
        }
    }
}
