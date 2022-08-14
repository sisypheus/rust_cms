// use std::fs::File;

use actix_easy_multipart::extractor::MultipartForm;
use actix_easy_multipart::{File, FromMultipart};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, Error as AWError, HttpResponse};
use serde::Deserialize;
use std::fs;
use std::io::Read;

use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
    services::{self, save_post as save_post_service},
};
use db::Pool;

#[derive(Deserialize, Debug)]
pub struct SavePostRequest {
    pub image: String,
}

#[derive(FromMultipart)]
pub struct Upload {
    description: String,
    post: File,
}

pub async fn login(
    db: web::Data<Pool>,
    params: web::Form<LoginRequest>,
    session: Session,
) -> Result<HttpResponse, AWError> {
    let user = services::login(db, params.into_inner()).await;

    match user {
        Ok(user) => {
            session.insert("user", &user)?;
            Ok(HttpResponse::Ok().json(user))
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

// pub async fn save_post(
//     db: web::Data<Pool>,
//     _user: User,
//     payload: Multipart,
// ) -> Result<HttpResponse, AWError> {
//     match save_post_service(db, payload).await {
//         Ok(_) => Ok(HttpResponse::Created().json({})),
//         Err(e) => Err(e),
//     }
// }

pub async fn save_post(
    db: web::Data<Pool>,
    _user: User,
    form: MultipartForm<Upload>,
) -> Result<HttpResponse, AWError> {
    // match save_post_service(db, form).await {
    //     Ok(_) => Ok(HttpResponse::Created().json({})),
    //     Err(e) => Err(e),
    // }
    let upload = form.post.file.path();
    let content = fs::read_to_string(&upload).unwrap();
    println!("{}", content);

    Ok(HttpResponse::Created().json({}))
}
