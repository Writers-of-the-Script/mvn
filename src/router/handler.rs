use std::sync::Arc;

use super::{docs::docs_handler, get::get_handler};
use crate::{auth::AnyAuth, cx::RouteContext, err::AxumResponse};
use anyhow::{Result, anyhow};
use axum::{
    extract::{Request, State},
    http::Method,
    response::Response,
};
use axum_extra::TypedHeader;

#[axum::debug_handler]
pub async fn route_handler(
    State(cx): State<Arc<RouteContext>>,
    auth: Option<TypedHeader<AnyAuth>>,
    req: Request,
) -> Result<Response, Response> {
    debug!("Request: Starting URI check...");

    let path = req.uri().path().to_string();

    if path.starts_with("/docs/") {
        debug!("Serving docs route!");

        let (_, parts) = path.split_once("/docs/").unwrap();

        let parts = parts
            .split("/")
            .map(|it| it.to_string())
            .collect::<Vec<_>>();

        debug!("Passing request to docs handler...");

        return docs_handler(cx, parts, auth).await;
    }

    debug!("Matching method...");

    match *req.method() {
        Method::GET => get_handler(cx, path, auth).await,

        Method::PUT => {
            debug!("Fetching token...");

            let token = auth
                .clone()
                .ok_or(anyhow!("Auth is required!"))
                .into_axum()?
                .get_token(&cx)
                .await
                .into_axum()?;

            debug!("Checking token...");

            if !token.can_write_to(&cx, &path).await.into_axum()? {
                return Err(Response::builder()
                    .status(403)
                    .body("403 Access Denied".into())
                    .into_axum()?);
            }

            debug!("Queueing upload...");

            cx.queue_upload(req.into_body(), &path).await;

            debug!("Passing back to GET handler...");

            get_handler(cx, path, auth).await
        }

        Method::DELETE => {
            debug!("Checking token...");

            let token = auth
                .ok_or(anyhow!("Auth is required!"))
                .into_axum()?
                .get_token(&cx)
                .await
                .into_axum()?;

            if !token.can_write_to(&cx, &path).await.into_axum()? {
                return Err(Response::builder()
                    .status(403)
                    .body("403 Access Denied".into())
                    .into_axum()?);
            }

            debug!("Checking file...");

            if cx.has_file(&path).await {
                debug!("Deleting file...");

                cx.delete_file(&path).await.into_axum()?;

                debug!("Re-indexing dirs...");

                cx
                    .index_dirs(&mut cx.pool.get().await.into_axum()?)
                    .await
                    .into_axum()?;
            }

            debug!("Sending response...");

            Ok(Response::builder()
                .status(200)
                .body("200 Ok".into())
                .into_axum()?)
        }

        _ => Err(Response::builder()
            .status(405)
            .body("405 Method Not Allowed".into())
            .into_axum()?),
    }
}
