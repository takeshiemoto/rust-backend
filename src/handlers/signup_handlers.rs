use crate::models::app_state::AppState;
use crate::models::signup_request::SignupRequest;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};

pub async fn signup(
    req: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let email = req.email.clone();
    let password = hash(req.password.clone(), DEFAULT_COST).unwrap();

    println!("email: {}, password: {}", email, password);

    sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2)")
        .bind(email.clone())
        .bind(password.clone())
        .execute(&app_state.pool)
        .await
        .unwrap();

    HttpResponse::Ok().body("signup_complete")
}

pub async fn signup_complete() -> impl Responder {
    HttpResponse::Ok().body("signup_complete")
}
