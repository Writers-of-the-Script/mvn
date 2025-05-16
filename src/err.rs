use anyhow::Result;
use axum::{body::Body, response::Response};
use std::fmt::Display;
use tokio::task::JoinError;

pub trait HasCode {
    /// Get the error code.
    fn code(&self) -> u16;
}

impl HasCode for anyhow::Error {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for std::io::Error {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for axum::Error {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for axum::http::Error {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for serde_json::Error {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for object_store::Error {
    fn code(&self) -> u16 {
        502
    }
}

impl HasCode for askama::Error {
    fn code(&self) -> u16 {
        502
    }
}

impl HasCode for zip::result::ZipError {
    fn code(&self) -> u16 {
        500
    }
}

impl HasCode for JoinError {
    fn code(&self) -> u16 {
        500
    }
}

pub trait AxumError: Display
where
    Self: Sized + HasCode,
{
    fn into_axum(self) -> Response {
        match Response::builder()
            .status(self.code())
            .body(Body::new(format!("{}", self)))
        {
            Ok(it) => it,
            Err(err) => Response::new(Body::new(format!("Could not create a response: {}", err))),
        }
    }
}

impl<T: Display + HasCode> AxumError for T {}

pub trait AxumResponse<T, E: AxumError> {
    fn into_axum(self) -> Result<T, Response>;
}

impl<T, E: AxumError> AxumResponse<T, E> for Result<T, E> {
    fn into_axum(self) -> Result<T, Response> {
        self.map_err(AxumError::into_axum)
    }
}
