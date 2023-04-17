use crate::models::app_state::AppState;
use actix_web::{web, Error, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(custom(
        code = "password",
        function = "validate_password",
        message = "Invalid password"
    ))]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8
        || password.contains(char::is_whitespace)
        || !password.chars().any(char::is_numeric)
        || !password.chars().any(char::is_uppercase)
    {
        return Err(ValidationError::new("Invalid password"));
    }
    Ok(())
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
