use axum::{http::header::CONTENT_TYPE, response::Response};

use crate::err::AxumResponse;

pub fn resp_404() -> Result<Response, Response> {
    Ok(Response::builder()
        .status(404)
        .header(CONTENT_TYPE, "text/plain")
        .body("404 Not Found".into())
        .into_axum()?)
}
