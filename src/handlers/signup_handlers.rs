use crate::models::app_state::AppState;
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use validator::Validate;

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

pub async fn signup(
    json: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = json.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let email = json.email.clone();
    let password = hash(json.password.clone(), DEFAULT_COST).unwrap();

    match sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2)")
        .bind(email.clone())
        .bind(password.clone())
        .execute(&app_state.pool)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn signup_complete() -> impl Responder {
    HttpResponse::Ok().body("signup_complete")
}
