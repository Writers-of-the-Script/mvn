use std::sync::Arc;

use crate::cx::RouteContext;
use admin::{auth_route, set_route_access};
use assets::{jbm_font_route, page_js_route, robots_txt_route};
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};
use force_auth::force_auth_middleware;
use handler::route_handler;
use logging::logging_middleware;
use tokens::{add_path_route, get_token_route, new_token_route};

pub mod access;
pub mod admin;
pub mod assets;
pub mod checks;
pub mod common;
pub mod docs;
pub mod force_auth;
pub mod get;
pub mod handler;
pub mod logging;
pub mod models;
pub mod request;
pub mod templates;
pub mod tokens;

pub fn build_router<S>(cx: Arc<RouteContext>) -> Router<S> {
    Router::new()
        .fallback(route_handler)
        .route("/api/auth", get(auth_route))
        .route("/api/token", put(new_token_route))
        .route("/api/token", post(add_path_route))
        .route("/api/token", get(get_token_route))
        .route("/api/access", post(set_route_access))
        .route("/assets/fonts/jetbrains-mono.woff2", get(jbm_font_route))
        .route("/assets/js/page.js", get(page_js_route))
        .route("/robots.txt", get(robots_txt_route))
        .layer(from_fn(logging_middleware))
        .layer(from_fn_with_state(Arc::clone(&cx), force_auth_middleware))
        .with_state(cx)
}
