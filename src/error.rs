use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("a custom error occured: {1}")]
    #[allow(dead_code)]
    Custom(StatusCode, String),

    #[error("Internal Server Error")]
    InternalServerError,

    #[error("a reqwest error occured: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("a ical parser error occured: {0}")]
    IcalParser(#[from] ical::parser::ParserError),

    #[error("{0}")]
    Eyre(#[from] eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Custom(status_code, message) => (
                status_code,
                Json(json!({
                    "message": message,
                })),
            )
                .into_response(),
            Self::Reqwest(_) | Self::IcalParser(_) | Self::InternalServerError | Self::Eyre(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Something went wrong"})),
            )
                .into_response(),
        }
    }
}
