use std::error::Error;
use std::fmt::Debug;

use crate::auth::User;
use actix_web::{
    error::{self, ErrorNotFound},
    web, Error as AWError,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub title: String,
    pub description: String,
    pub content: String,
    pub image: String,
}

pub async fn get_all_posts(pool: &Pool) -> Result<Vec<Post>, AWError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let posts = get_posts_from_db(conn)?;
    Ok(posts)
}

fn get_posts_from_db(conn: Connection) -> Result<Vec<Post>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT * FROM posts")?;
    let posts_iter = stmt.query_map([], |row| {
        Ok(Post {
            title: row.get(1)?,
            description: row.get(2)?,
            image: row.get(3)?,
            content: row.get(4)?,
        })
    })?;
    let mut posts = vec![];
    for post in posts_iter {
        posts.push(post?);
    }
    Ok(posts)
}

pub async fn save_post(pool: &Pool, post: Post) -> Result<(), AWError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let status = save_post_to_db(&conn, post).await;
    match status {
        Ok(_) => (),
        Err(e) => return Err(AWError::from(e)),
    }
    Ok(())
}

pub async fn save_post_to_db(conn: &Connection, post: Post) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "INSERT INTO posts (title, description, image, content) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let params = params![post.title, post.description, post.image, post.content];
    stmt.execute(params)?;
    Ok(())
}

pub async fn get_post(db: &Pool, id: i32) -> Result<Post, AWError> {
    let pool = db.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;
    let post = get_post_from_db(&conn, id).await;
    match post {
        Some(post) => Ok(post),
        None => Err(ErrorNotFound("Post not found").into()),
    }
}

async fn get_post_from_db(conn: &Connection, id: i32) -> Option<Post> {
    let stmt = conn.prepare("SELECT * FROM posts WHERE id = ?1 LIMIT 1");

    let mut stmt = match stmt {
        Ok(stmt) => stmt,
        Err(_) => return None,
    };

    let post = stmt.query_row(&[&id], |row| {
        Ok(Post {
            title: row.get(1)?,
            description: row.get(2)?,
            image: row.get(3)?,
            content: row.get(4)?,
        })
    });

    match post {
        Ok(post) => return Some(post),
        Err(_) => return None,
    }
}

pub async fn find_user_by_username(pool: &Pool, username: String) -> Result<User, AWError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let user = get_user_by_username(conn, username)?;
    Ok(user)
}

fn get_user_by_username(conn: Connection, username: String) -> Result<User, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT * FROM users WHERE username = ?")?;
    let user = stmt.query_row(&[&username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
        })
    })?;
    Ok(user)
}
