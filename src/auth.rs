use std::{error::Error, pin::Pin};

use actix_session::SessionExt;
use actix_web::error::ErrorUnauthorized;
use actix_web::Result;
use actix_web::{FromRequest, HttpRequest};
// use anyhow::Ok;
// use anyhow::Result;
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
    type Error = Box<dyn Error>;
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
            Err(_) => return Box::pin(async { Err(ErrorUnauthorized("Unauthorized").into()) }),
        };

        if user.is_some() {
            return Box::pin(async { Ok(user.unwrap()) });
        } else {
            return Box::pin(async { Err(ErrorUnauthorized("Unauthorized").into()) });
        }
    }
}
