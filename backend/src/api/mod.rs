use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::Error;
use thiserror::Error;
use tokio::time::error::Elapsed;
use log::error;

//pub mod delete;
//pub mod downloaded;
pub mod search;
//pub mod upload;
//pub mod upvote;
//pub mod usersongs;
//pub mod signin;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Ratelimited")]
    Ratelimited(),
    #[error("Authentication error")]
    AuthError(Error),
    #[error("Database error")]
    DatabaseError(Error),
    #[error("Deserialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Already upvoted!")]
    AlreadyUpvoted(),
    #[error("Already downloaded!")]
    AlreadyDownloaded(),
    #[error("Expected a multi-part form!")]
    ArgumentError(),
    #[error("Unknown archive type, please submit a zip or rar!")]
    ArchiveTypeError(),
    #[error("Error with multi-part form!")]
    KnownArgumentError(Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Zip error, please confirm your beatmap file is correct and contains all needed files")]
    ZipError(Error),
    #[error("Zip download error")]
    ZipDownloadError(#[from] serenity::Error),
    #[error("Invalid song name")]
    SongNameError(#[from] serde_urlencoded::ser::Error),
    #[error("Served timed out reading archive")]
    TimeoutError(#[from] Elapsed),
    #[error("You do not have permission to perform this action")]
    PermissionError(),
}

impl APIError {
    pub fn get_code(&self) -> StatusCode {
        match self {
            APIError::Ratelimited() => StatusCode::TOO_MANY_REQUESTS,
            APIError::AuthError(_)
            | APIError::AlreadyUpvoted()
            | APIError::AlreadyDownloaded()
            | APIError::ArgumentError()
            | APIError::KnownArgumentError(_)
            | APIError::SongNameError(_)
            | APIError::ArchiveTypeError()
            | APIError::PermissionError() => StatusCode::BAD_REQUEST,
            APIError::DatabaseError(_)
            | APIError::SerdeError(_)
            | APIError::ZipError(_)
            | APIError::IOError(_)
            | APIError::TimeoutError(_)
            | APIError::ZipDownloadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn database_error<E: Into<Error>>(error: E) -> APIError {
        APIError::DatabaseError(error.into())
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        self.get_code()
    }

    fn error_response(&self) -> HttpResponse {
        error!("Error: {self:?}");
        HttpResponse::with_body(self.status_code(), self.to_string()).map_into_boxed_body()
    }
}
