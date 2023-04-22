use crate::errors::{APILayerError, AppError};
use crate::models::app_state::AppState;
use crate::models::user::{User, UserId};
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lettre::address::AddressError;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
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

    const EXPIRATION: u64 = 3600;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?
        .as_secs();

    let claims = Claims {
        sub: user.id.0.to_string(),
        exp: now + EXPIRATION,
    };

    let secret_key = "secret";
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    let client_url = env::var("CLIENT_URL")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;
    let from = env::var("EMAIL_FROM")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;
    let body = format!(
        "Please click on the URL to authenticate .\n\n{}/signup/verify?token={}",
        client_url, token
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

pub async fn signup_verify(
    req: web::Query<SignupVerifyQuery>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    req.validate().map_err(AppError::Validation)?;

    let claims = decode::<Claims>(
        &req.token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(APILayerError::new(e.to_string())))?;

    sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
        .bind(claims.claims.sub.parse::<i32>().unwrap())
        .execute(&app_state.pool)
        .await
        .map_err(AppError::DatabaseQuery)?;

    Ok(HttpResponse::Ok())
}
