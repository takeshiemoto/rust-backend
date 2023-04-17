use crate::models::app_state::AppState;
use actix_web::{web, Error, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 4, message = "Password must be at least 4 characters long"))]
    pub password: String,
}

pub async fn signup(
    req: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(e));
    }

    let email = req.email.clone();
    let password = hash(req.password.clone(), DEFAULT_COST).unwrap();

    sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2)")
        .bind(email.clone())
        .bind(password.clone())
        .execute(&app_state.pool)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().finish())
}

pub async fn signup_complete() -> impl Responder {
    HttpResponse::Ok().body("signup_complete")
}
