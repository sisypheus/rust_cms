use std::{error::Error, fs, io::Write};

use crate::{
    auth::{LoginRequest, User},
    db::{self, Post},
};

use actix_multipart::Multipart;
use actix_web::{error::ErrorUnauthorized, web, Error as AWError};
use db::Pool;
use futures_util::TryStreamExt;
use uuid::Uuid;

fn file_to_vec(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    Ok(lines)
}

pub async fn save_post(db: web::Data<Pool>, mut payload: Multipart) -> Result<(), AWError> {
    let mut content: Vec<String> = vec![];
    let mut image: String = "".to_string();
    while let Some(mut field) = payload.try_next().await? {
        let mut is_image: bool = false;
        let content_disposition = field.content_disposition();

        match &content_disposition.parameters.get(0) {
            Some(param) => match param.as_name() {
                Some(image) => {
                    if image == "image" {
                        is_image = true;
                    }
                }
                None => {}
            },
            None => {}
        }

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let filepath = format!("/tmp/{filename}");

        let mut f = {
            let filepath = filepath.clone();
            web::block(move || std::fs::File::create(&filepath)).await??
        };

        while let Some(chunk) = field.try_next().await? {
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }

        let data = file_to_vec(&filepath)?;

        if is_image {
            image = {
                let mut string: String = "".to_string();
                for line in data {
                    string.push_str(&line);
                }
                string
            }
        } else {
            content = data;
        }

        fs::remove_file(format!("/tmp/{filename}")).expect("Unable to delete file");
    }
    let post = Post {
        title: if content.len() > 0 {
            content.remove(0)
        } else {
            "".to_string()
        },
        description: if content.len() > 0 {
            content.remove(0)
        } else {
            "".to_string()
        },
        content: content.join("\n"),
        image,
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
