use actix_web::{HttpResponse, Responder};

pub async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("get_users")
}

pub async fn get_users_by_id() -> impl Responder {
    HttpResponse::Ok().body("get_users_by_id")
}

pub async fn create_users() -> impl Responder {
    HttpResponse::Ok().body("create_users")
}

pub async fn update_users() -> impl Responder {
    HttpResponse::Ok().body("update_users")
}

pub async fn delete_users() -> impl Responder {
    HttpResponse::Ok().body("delete_user")
}
