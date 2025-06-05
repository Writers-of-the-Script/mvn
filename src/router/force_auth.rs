use crate::{auth::AnyAuth, cx::RouteContext};
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{
        Request,
        header::{CONTENT_TYPE, WWW_AUTHENTICATE},
    },
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use std::{collections::HashMap, sync::Arc};
use url::Url;

pub async fn force_auth_middleware(
    State(cx): State<Arc<RouteContext>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    debug!("Getting query...");

    let query = req
        .uri()
        .query()
        .map(|it| it.to_string())
        .unwrap_or_default();

    debug!("Parsing URL...");

    let url = Url::parse(&format!("http://placeholder/?{}", query)).unwrap();

    debug!("Collecting query...");

    let query = HashMap::<_, _>::from_iter(
        url.query_pairs()
            .map(|(a, b)| (a.to_string(), b.to_string())),
    );

    debug!("Creating response...");

    let auth_resp = Response::builder()
        .status(401)
        .header(WWW_AUTHENTICATE, "Basic realm=\"Maven\", charset=\"UTF-8\"")
        .header(CONTENT_TYPE, "text/plain")
        .body("Authorizing...".into())
        .unwrap();

    debug!("Checking query...");

    if query.get("force_auth").is_some_and(|it| it == "1") {
        debug!("Forced! Getting parts...");

        let (mut parts, body) = req.into_parts();

        debug!("Getting auth header...");

        let auth = Option::<TypedHeader<AnyAuth>>::from_request_parts(&mut parts, &cx)
            .await
            .unwrap_or_default();

        debug!("Building request...");

        let req = Request::from_parts(parts, body);

        debug!("Checking auth...");

        if let Some(auth) = auth {
            if let Ok(_) = auth.get_token(&cx).await {
                debug!("Passing through...");

                next.run(req).await
            } else {
                auth_resp
            }
        } else {
            auth_resp
        }
    } else {
        debug!("Passing through...");

        next.run(req).await
    }
}
