use crate::errors::Error;
use crate::models::app_state::AppState;
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

pub async fn signup(
    json: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    json.validate().map_err(Error::ValidationError)?;

    let email = json.email.clone();
    let password = hash(json.password.clone(), DEFAULT_COST).unwrap();

    sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2)")
        .bind(email.clone())
        .bind(password.clone())
        .execute(&app_state.pool)
        .await
        .map_err(Error::DatabaseQueryError)?;

    let expiration = 3600;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: email,
        exp: now + expiration,
    };

    let secret_key = "secret";
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .unwrap();

    let signup_response = SignupResponse { token };
    Ok(HttpResponse::Ok().json(signup_response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupVerifyQuery {
    pub token: String,
}

#[instrument]
pub async fn signup_verify(query: web::Query<SignupVerifyQuery>) -> impl Responder {
    info!(query.token);
    HttpResponse::Ok().body("signup_complete")
}
