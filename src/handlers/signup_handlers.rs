use crate::errors::Error;
use crate::models::app_state::AppState;
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::env;
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
struct Claims {
    sub: String,
    exp: u64,
}

pub async fn signup(
    json: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    json.validate().map_err(Error::Validation)?;

    let email = json.email.clone();
    let password = hash(json.password.clone(), DEFAULT_COST).map_err(|_| Error::Internal)?;

    sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2)")
        .bind(email.clone())
        .bind(password.clone())
        .execute(&app_state.pool)
        .await
        .map_err(Error::DatabaseQuery)?;

    const EXPIRATION: u64 = 30;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::Internal)?
        .as_secs();

    let claims = Claims {
        sub: email.clone(),
        exp: now + EXPIRATION,
    };

    let secret_key = "secret";
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|_| Error::Internal)?;

    let client_url = env::var("CLIENT_URL").map_err(|_| Error::Internal)?;
    let from = env::var("EMAIL_FROM").map_err(|_| Error::Internal)?;
    let body = format!(
        "Please click on the URL to authenticate .\n\n{}/signup/verify?token={}",
        client_url, token
    );
    let message = Message::builder()
        .from(from.parse().map_err(|_| Error::Internal)?)
        .to(email.parse().map_err(|_| Error::Internal)?)
        .subject("Welcome!")
        .body(body)
        .map_err(|_| Error::Internal)?;

    let api_key = env::var("SENDGRID_API_KEY").map_err(|_| Error::Internal)?;
    let creds = Credentials::new("apikey".to_string(), api_key);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")
        .map_err(|_| Error::Internal)?
        .credentials(creds)
        .build();

    mailer.send(&message).map_err(|_| Error::Internal)?;

    Ok(HttpResponse::Ok())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupVerifyQuery {
    pub token: String,
}

pub async fn signup_verify() -> impl Responder {
    HttpResponse::Ok().body("signup_complete")
}
