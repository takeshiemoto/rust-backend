use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};
use serde::Serialize;
use std::borrow::Cow;
use tracing::log::error;

#[derive(Debug, Display, Error)]
pub enum AppError {
    Deserialization(APILayerError),
    Validation(validator::ValidationErrors),
    DatabaseQuery(sqlx::Error),
    Unauthorized(APILayerError),
    Internal(APILayerError),
}

#[derive(Debug, Display, Error)]
pub struct APILayerError {
    pub message: String,
}

impl APILayerError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Vec<validator::ValidationErrors>,
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Validation(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Deserialization(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::DatabaseQuery(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Deserialization(e) => {
                error!("Deserialization error: {:?}", self);
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "DESERIALIZATION_ERROR".to_string(),
                    message: e.message.clone(),
                    details: vec![],
                })
            }
            AppError::Validation(e) => {
                error!("Validation error: {:?}", self);
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Validation error".to_string(),
                    details: vec![e.clone()],
                })
            }
            AppError::DatabaseQuery(error) => match error {
                sqlx::Error::Database(db_error)
                    if db_error.code() == Some(Cow::Borrowed("23505")) =>
                {
                    HttpResponse::Conflict().json(ErrorResponse {
                        code: "DUPLICATE_ERROR".to_string(),
                        message: error.to_string(),
                        details: vec![],
                    })
                }
                _ => HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "INTERNAL_ERROR".to_string(),
                    message: error.to_string(),
                    details: vec![],
                }),
            },
            AppError::Unauthorized(e) => {
                error!("Unauthorized error: {:?}", self);
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "UNAUTHORIZED_ERROR".to_string(),
                    message: e.message.clone(),
                    details: vec![],
                })
            }
            AppError::Internal(error) => HttpResponse::InternalServerError().json(ErrorResponse {
                code: "INTERNAL_ERROR".to_string(),
                message: error.message.clone(),
                details: vec![],
            }),
        }
    }
}
