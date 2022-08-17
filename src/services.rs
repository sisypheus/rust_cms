use std::{fs, io::Write};

use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
};

use actix_multipart::Multipart;
use actix_web::{error::ErrorUnauthorized, web, Error as AWError};
use db::Pool;
use futures_util::{StreamExt, TryStreamExt};
use uuid::Uuid;

pub async fn save_post(db: web::Data<Pool>, mut payload: Multipart) -> Result<(), AWError> {
    let mut title = String::new();
    let mut description = String::new();
    let mut image = String::new();
    let mut content = String::new();
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item?;

        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let filepath = format!("./tmp/{filename}");

        let mut f = {
            let filepath = filepath.clone();
            web::block(|| std::fs::File::create(filepath)).await??
        };

        while let Some(chunk) = field.try_next().await? {
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
        let value = fs::read_to_string(&filepath)?;

        match field.name() {
            "title" => title = value,
            "description" => description = value,
            "image" => image = value,
            "content" => content = value,
            _ => (),
        }
        fs::remove_file(filepath)?;
    }
    let post = Post {
        title,
        description,
        image,
        content,
    };
    db::save_post(&db, post).await?;
    Ok(())
}

pub async fn login(db: web::Data<Pool>, form: LoginRequest) -> Result<User, AWError> {
    let user: User = db::find_user_by_username(&db, form.username).await?;

    if argon2::verify_encoded(&user.password, form.password.as_bytes()).unwrap() {
        Ok(user)
    } else {
        Err(ErrorUnauthorized("Invalid credentials"))
    }
}
