use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};

use tracing::log::error;

#[derive(Debug, Display, Error)]
pub enum Error {
    Validation(validator::ValidationErrors),
    DatabaseQuery(sqlx::Error),
    Internal,
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
            Error::Validation(e) => HttpResponse::BadRequest().json(e),
            Error::DatabaseQuery(e) => {
                error!("Database query error: {}", e);
                HttpResponse::InternalServerError().body("Database query error")
            }
            Error::Internal => HttpResponse::InternalServerError().finish(),
        }
    }
}
