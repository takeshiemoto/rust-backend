mod handlers;
mod models;

use crate::handlers::user_handlers::{
    create_users, delete_users, get_users, get_users_by_id, update_users,
};
use std::env;

use crate::handlers::signup_handlers::{signup, signup_complete};
use crate::models::app_state::AppState;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await.unwrap();
    let app_state = AppState { pool };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(
                web::scope("/api").service(
                    web::scope("/v1")
                        .service(
                            web::scope("/users")
                                .route("", web::get().to(get_users))
                                .route("", web::post().to(create_users))
                                .route("/{id}", web::get().to(get_users_by_id))
                                .route("/{id}", web::put().to(update_users))
                                .route("/{id}", web::delete().to(delete_users)),
                        )
                        .service(
                            web::scope("/signup")
                                .route("", web::post().to(signup))
                                .route("/complete", web::get().to(signup_complete)),
                        ),
                ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
