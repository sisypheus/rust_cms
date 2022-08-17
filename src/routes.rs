use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, Error as AWError, HttpResponse};
use serde::Deserialize;

use crate::auth::LoginResponse;
use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
    services::{self, save_post as save_post_service},
};
use db::Pool;

#[derive(Deserialize, Debug)]
pub struct SavePostRequest {
    pub title: String,
    pub description: String,
    pub image: String,
}

pub async fn protected(_pool: web::Data<Pool>, _user: User) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::Ok().body("protected"))
}

pub async fn login(
    db: web::Data<Pool>,
    params: web::Json<LoginRequest>,
    session: Session,
) -> Result<HttpResponse, AWError> {
    let user = services::login(db, params.into_inner()).await;

    match user {
        Ok(user) => {
            session.insert("user", &user)?;
            Ok(HttpResponse::Ok().json(LoginResponse::new(&user)))
        }
        Err(_) => Ok(HttpResponse::Unauthorized().json({})),
    }
}

pub async fn get_all_posts(db: web::Data<Pool>) -> Result<HttpResponse, AWError> {
    let result: Vec<Post> = db::get_all_posts(&db).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn get_post(db: web::Data<Pool>, id: web::Path<i32>) -> Result<HttpResponse, AWError> {
    let result: Post = db::get_post(&db, id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn save_post(
    db: web::Data<Pool>,
    _user: User,
    payload: Multipart,
) -> Result<HttpResponse, AWError> {
    match save_post_service(db, payload).await {
        Ok(_) => Ok(HttpResponse::Created().json({})),
        Err(e) => Err(e),
    }
}
