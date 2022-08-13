use std::{error::Error, fs, io::Write};

use crate::db::{self, Post};

use actix_multipart::Multipart;
use actix_web::{web, Error as AWError};
use db::Pool;
use futures_util::TryStreamExt;
use uuid::Uuid;

fn file_to_vec(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    Ok(lines)
}

pub async fn save_posts(db: web::Data<Pool>, mut payload: Multipart) -> Result<(), AWError> {
    let mut posts: Vec<Post> = Vec::new();

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

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

        let data = file_to_vec(&filepath);

        match data {
            Ok(mut lines) => {
                let post = Post {
                    title: lines.remove(0),
                    content: lines.join("\n"),
                };
                posts.push(post);
            }
            Err(e) => return Err(AWError::from(e)),
        }
        fs::remove_file(format!("/tmp/{filename}")).expect("Unable to delete file");
    }
    db::save_posts(&db, posts).await?;
    Ok(())
}
