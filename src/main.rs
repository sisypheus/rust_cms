use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::middleware;
use actix_web::{cookie::Key, web, App, HttpServer};
use argon2::{self, Config};
use db::Pool;
use r2d2_sqlite::{self, SqliteConnectionManager};
use std::env;
use std::error::Error;

mod auth;
mod db;
mod routes;
mod services;

async fn create_admin(db: web::Data<Pool>) -> Result<(), Box<dyn Error>> {
    let conn = db.get()?;
    let username = env::var("ADMIN_USERNAME")?;
    let password = env::var("ADMIN_PASSWORD")?;
    let salt = env::var("SALT")?;
    let config = Config::default();

    let hash = argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &config)
        .expect("Failed to hash password");
    let sql = format!(
        "INSERT OR IGNORE INTO users (username, password) VALUES ('{}', '{}')",
        username, hash
    );
    let mut stmt = conn.prepare(&sql)?;
    stmt.execute([])?;
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::fs::create_dir_all("./tmp")?;
    let manager = SqliteConnectionManager::file("db/db");
    let pool = Pool::new(manager).unwrap();
    let cookie_secure: bool = env::var("COOKIE_SECURE")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();

    env::set_var("RUST_LOG", "debug");

    match create_admin(web::Data::new(pool.clone())).await {
        Ok(_) => {}
        Err(e) => println!("Error creating admin: {}", e),
    }

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(cookie_secure)
                    .build(),
            )
            .service(web::resource("/protected").route(web::get().to(routes::protected)))
            .service(
                web::scope("/posts")
                    .service(web::resource("").route(web::get().to(routes::get_all_posts)))
                    .service(web::resource("/save").route(web::post().to(routes::save_post)))
                    .service(web::resource("/{id}").route(web::get().to(routes::get_post))),
            )
            .service(web::resource("/login").route(web::post().to(routes::login)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
