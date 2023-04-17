use actix_web::{web, App, HttpResponse, HttpServer, Responder};

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

async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("get_users")
}

async fn get_users_by_id() -> impl Responder {
    HttpResponse::Ok().body("get_users_by_id")
}

async fn create_users() -> impl Responder {
    HttpResponse::Ok().body("create_users")
}

async fn update_users() -> impl Responder {
    HttpResponse::Ok().body("update_users")
}

async fn delete_users() -> impl Responder {
    HttpResponse::Ok().body("delete_user")
}
