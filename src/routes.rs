use actix_multipart::Multipart;
use std::{fs, io::Write};

use actix_session::Session;
use actix_web::{web, Error as AWError, HttpResponse};
use futures_util::TryStreamExt as _;
use uuid::Uuid;

use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
};
use db::Pool;

pub async fn login(
    db: web::Data<Pool>,
    params: web::Form<LoginRequest>,
    session: Session,
) -> Result<HttpResponse, AWError> {
    let user: User = db::find_user_by_username(&db, params.username.clone()).await?;

    if argon2::verify_encoded(&user.password, params.password.as_bytes()).unwrap() {
        session.insert("user", &user)?;
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Unauthorized().json({}))
    }
}

#[allow(unused_variables)]
pub async fn get_all_posts(db: web::Data<Pool>, user: User) -> Result<HttpResponse, AWError> {
    let result: Vec<Post> = db::execute(&db).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn save_posts(
    db: web::Data<Pool>,
    _user: User,
    mut payload: Multipart,
) -> Result<HttpResponse, AWError> {
    let mut posts: Vec<Post> = vec![];
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let filepath = format!("/tmp/{filename}");

        let mut f = web::block(|| std::fs::File::create(filepath)).await??;

        while let Some(chunk) = field.try_next().await? {
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
        let data = fs::read_to_string(format!("/tmp/{filename}")).expect("Unable to read file");

        match data.is_empty() {
            true => {
                let post = Post {
                    id: 0,
                    title: "".to_string(),
                    content: "".to_string(),
                };
                posts.push(post);
            }
            false => {
                let post = Post {
                    id: 0,
                    title: "".to_string(),
                    content: data,
                };
                posts.push(post);
            }
        }
        fs::remove_file(format!("/tmp/{filename}")).expect("Unable to delete file");
    }
    db::save_posts(&db, posts).await?;

    Ok(HttpResponse::Ok().json({}))
}
