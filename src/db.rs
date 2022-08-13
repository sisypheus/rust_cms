use std::error::Error;

use crate::auth::User;
use actix_web::{error, web, Error as AWError};
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub title: String,
    pub content: String,
}

pub async fn execute(pool: &Pool) -> Result<Vec<Post>, AWError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let posts = get_all_posts(conn)?;
    Ok(posts)
}

fn get_all_posts(conn: Connection) -> Result<Vec<Post>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT * FROM posts")?;
    let posts_iter = stmt.query_map([], |row| {
        Ok(Post {
            title: row.get(1)?,
            content: row.get(2)?,
        })
    })?;
    let mut posts = vec![];
    for post in posts_iter {
        let post = post?;
        posts.push(post);
    }
    Ok(posts)
}

pub async fn save_posts(pool: &Pool, posts: Vec<Post>) -> Result<(), AWError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;
    for post in posts {
        let status = save_post(&conn, post).await;
        match status {
            Ok(_) => (),
            Err(e) => return Err(AWError::from(e)),
        }
    }
    Ok(())
}

pub async fn save_post(conn: &Connection, post: Post) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare("INSERT INTO posts (title, content) VALUES (?1, ?2)")?;
    stmt.execute(&[&post.title, &post.content])?;
    Ok(())
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
