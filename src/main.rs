mod handlers;

use crate::handlers::user_handlers::{
    create_users, delete_users, get_users, get_users_by_id, update_users,
};

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/api").service(
                web::scope("/v1").service(
                    web::scope("/users")
                        .route("", web::get().to(get_users))
                        .route("", web::post().to(create_users))
                        .route("/{id}", web::get().to(get_users_by_id))
                        .route("/{id}", web::put().to(update_users))
                        .route("/{id}", web::delete().to(delete_users)),
                ),
            ),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
