use crate::errors::{APILayerError, AppError};
use crate::models::app_state::AppState;
use crate::models::user::{User, UserId};
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use lettre::address::AddressError;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::Uuid;
use sqlx::Row;
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

#[derive(Debug)]
pub struct Token(pub Uuid);

pub async fn signup(
    json: web::Json<SignupRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    json.validate().map_err(AppError::Validation)?;

    let email = json.email.clone();
    let password = hash(json.password.clone(), DEFAULT_COST)
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    let user = sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
        .bind(email.clone())
        .bind(password.clone())
        .map(|row: PgRow| User {
            id: UserId(row.get("id")),
            email: email.clone(),
            password: password.clone(),
        })
        .fetch_one(&app_state.pool)
        .await
        .map_err(AppError::DatabaseQuery)?;

    let token =
        sqlx::query("INSERT INTO email_verification_tokens (user_id, expires_at) VALUES ($1, $2) RETURNING token")
            .bind(user.id.0)
            .bind(Utc::now() + Duration::hours(24))
            .map(|row: PgRow| Token(row.get("token")))
            .fetch_one(&app_state.pool)
            .await
            .map_err(AppError::DatabaseQuery)?;

    let client_url = env::var("CLIENT_URL")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;
    let from = env::var("EMAIL_FROM")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;
    let body = format!(
        "Please click on the URL to authenticate .\n\n{}/signup/verify?token={}",
        client_url, token.0
    );
    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|e: AddressError| AppError::Internal(APILayerError::new(e.to_string())))?,
        )
        .to(email
            .parse()
            .map_err(|e: AddressError| AppError::Internal(APILayerError::new(e.to_string())))?)
        .subject("Welcome!")
        .body(body)
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    let api_key = env::var("SENDGRID_API_KEY")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;
    let creds = Credentials::new("apikey".to_string(), api_key);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?
        .credentials(creds)
        .build();

    mailer
        .send(&message)
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    Ok(HttpResponse::Ok())
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignupVerifyQuery {
    #[validate(length(min = 1, message = "token is required"))]
    pub token: String,
}

pub async fn signup_verify(req: web::Query<SignupVerifyQuery>) -> Result<impl Responder, AppError> {
    req.validate().map_err(AppError::Validation)?;

    Ok(HttpResponse::Ok())
}
