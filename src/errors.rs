use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};
use serde::Serialize;

use tracing::log::error;

#[derive(Debug, Display, Error)]
pub enum Error {
    Validation(validator::ValidationErrors),
    DatabaseQuery(sqlx::Error),
    Internal,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Vec<validator::ValidationErrors>,
}

impl error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::Validation(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Error::DatabaseQuery(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Validation(e) => {
                error!("Validation error: {:?}", self);
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "validation_error".to_string(),
                    message: "Validation error".to_string(),
                    details: vec![e.clone()],
                })
            }
            Error::DatabaseQuery(e) => {
                error!("Database query error: {:?}", e);
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "internal_error".to_string(),
                    message: e.to_string(),
                    details: vec![],
                })
            }
            _ => {
                error!("Internal error: {:?}", self);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "internal_error".to_string(),
                    message: "Internal error".to_string(),
                    details: vec![],
                })
            }
        }
    }
}
