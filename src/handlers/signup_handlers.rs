use crate::errors::{APILayerError, AppError};
use crate::mailer::send_mail;
use crate::models::app_state::AppState;
use crate::models::user::{User, UserId};
use crate::validators::password_validator::validate_password;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use lettre::address::AddressError;
use lettre::Message;
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

    let mut transaction = app_state
        .pool
        .begin()
        .await
        .map_err(AppError::DatabaseQuery)?;

    let user = sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
        .bind(email.clone())
        .bind(password.clone())
        .map(|row: PgRow| User {
            id: UserId(row.get("id")),
            email: email.clone(),
            password: password.clone(),
        })
        .fetch_one(&mut transaction)
        .await
        .map_err(AppError::DatabaseQuery)?;

    let token =
        sqlx::query("INSERT INTO email_verification_tokens (user_id, expires_at) VALUES ($1, $2) RETURNING token")
            .bind(user.id.0)
            .bind(Utc::now() + Duration::hours(24))
            .map(|row: PgRow| Token(row.get("token")))
            .fetch_one(&mut transaction)
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

    if let Err(e) = send_mail(&message).await {
        transaction
            .rollback()
            .await
            .map_err(AppError::DatabaseQuery)?;

        return Err(AppError::Internal(APILayerError::new(e.to_string())));
    }

    transaction
        .commit()
        .await
        .map_err(AppError::DatabaseQuery)?;

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

    let mut transaction = app_state
        .pool
        .begin()
        .await
        .map_err(AppError::DatabaseQuery)?;

    let user_id = sqlx::query(
        "SELECT user_id FROM email_verification_tokens WHERE token = $1::uuid AND expires_at > CURRENT_TIMESTAMP",
    )
    .bind(req.token.clone())
    .map(|row: PgRow| UserId(row.get("user_id")))
    .fetch_optional(&mut transaction)
    .await
    .map_err(AppError::DatabaseQuery)?;

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => {
            return Err(AppError::Unauthorized(APILayerError::new(
                "Token has expired.".to_string(),
            )));
        }
    };

    let user = sqlx::query(
        "UPDATE users SET email_verified = true WHERE id = $1 RETURNING id, email, password",
    )
    .bind(user_id.0)
    .map(|row: PgRow| User {
        id: UserId(row.get("id")),
        email: row.get("email"),
        password: row.get("password"),
    })
    .fetch_one(&mut transaction)
    .await
    .map_err(AppError::DatabaseQuery)?;

    println!("{:?}", user);

    let from = env::var("EMAIL_FROM")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    let client_url = env::var("CLIENT_URL")
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    let body = format!(
        "Please login from the following URL .\n\n{}/signin",
        client_url
    );

    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|e: AddressError| AppError::Internal(APILayerError::new(e.to_string())))?,
        )
        .to(user
            .email
            .parse()
            .map_err(|e: AddressError| AppError::Internal(APILayerError::new(e.to_string())))?)
        .subject("Your registration has been completed.!")
        .body(body)
        .map_err(|e| AppError::Internal(APILayerError::new(e.to_string())))?;

    if let Err(e) = send_mail(&message).await {
        transaction
            .rollback()
            .await
            .map_err(AppError::DatabaseQuery)?;

        return Err(AppError::Internal(APILayerError::new(e.to_string())));
    }

    transaction
        .commit()
        .await
        .map_err(AppError::DatabaseQuery)?;

    Ok(HttpResponse::Ok())
}
