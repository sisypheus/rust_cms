use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, Error as AWError, HttpResponse};

use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
    services::save_posts as save_posts_service,
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

pub async fn get_all_posts(db: web::Data<Pool>, _user: User) -> Result<HttpResponse, AWError> {
    let result: Vec<Post> = db::execute(&db).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn save_posts(
    db: web::Data<Pool>,
    _user: User,
    payload: Multipart,
) -> Result<HttpResponse, AWError> {
    match save_posts_service(db, payload).await {
        Ok(_) => Ok(HttpResponse::Created().json({})),
        Err(e) => Err(e),
    }
}
