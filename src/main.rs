use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, get, middleware, web, App, HttpServer, Responder};
use argon2::{self, Config};
use db::Pool;
use r2d2_sqlite::{self, SqliteConnectionManager};
use std::env;
use std::error::Error;

mod auth;
mod db;
mod routes;
mod services;

#[get("/hello")]
async fn hello() -> impl Responder {
    "Hello world!"
}

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
    let manager = SqliteConnectionManager::file("db/db");
    let pool = Pool::new(manager).unwrap();

    match create_admin(web::Data::new(pool.clone())).await {
        Ok(_) => {}
        Err(e) => println!("Error creating admin: {}", e),
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .build(),
            )
            .service(
                web::scope("/posts")
                    .service(web::resource("").route(web::get().to(routes::get_all_posts)))
                    .service(web::resource("/save").route(web::post().to(routes::save_posts))),
            )
            .service(web::resource("/login").route(web::post().to(routes::login)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
