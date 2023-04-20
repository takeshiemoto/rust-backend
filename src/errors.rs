use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};

use tracing::log::error;

#[derive(Debug, Display, Error)]
pub enum Error {
    ValidationError(validator::ValidationErrors),
    DatabaseQueryError(sqlx::Error),
    InternalServerError,
}

impl error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Error::DatabaseQueryError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::ValidationError(e) => HttpResponse::BadRequest().json(e),
            Error::DatabaseQueryError(e) => {
                error!("Database query error: {}", e);
                HttpResponse::InternalServerError().body("Database query error")
            }
            Error::InternalServerError => HttpResponse::InternalServerError().finish(),
        }
    }
}
