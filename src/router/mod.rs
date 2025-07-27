use std::sync::Arc;

use crate::cx::RouteContext;
use admin::{
    admin_dashboard_route, auth_route, delete_route_access, login_route, set_route_access,
};
use assets::{
    copy_svg_route, jbm_font_route, page_js_route, plus_svg_route, robots_txt_route,
    trash_svg_route,
};
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, put},
};
use force_auth::force_auth_middleware;
use handler::route_handler;
use logging::logging_middleware;
use tokens::{
    add_path_route, delete_path_route, delete_token_route, get_token_paths_route, get_token_route,
    new_token_route,
};

pub mod access;
pub mod admin;
pub mod assets;
pub mod checks;
pub mod common;
pub mod dash;
pub mod docs;
pub mod force_auth;
pub mod get;
pub mod handler;
pub mod logging;
pub mod models;
pub mod request;
pub mod stats;
pub mod templates;
pub mod tokens;

pub fn build_router<S>(cx: Arc<RouteContext>) -> Router<S> {
    Router::new()
        .fallback(route_handler)
        .route("/api/auth", get(auth_route))
        .route("/api/token", get(get_token_route))
        .route("/api/token", put(new_token_route))
        .route("/api/token", delete(delete_token_route))
        .route("/api/token/paths", get(get_token_paths_route))
        .route("/api/token/paths", put(add_path_route))
        .route("/api/token/paths", delete(delete_path_route))
        .route("/api/access", put(set_route_access))
        .route("/api/access", delete(delete_route_access))
        .route("/assets/fonts/jetbrains-mono.woff2", get(jbm_font_route))
        .route("/assets/js/page.js", get(page_js_route))
        .route("/robots.txt", get(robots_txt_route))
        .route("/admin", get(admin_dashboard_route))
        .route("/admin/assets/copy.svg", get(copy_svg_route))
        .route("/admin/assets/plus.svg", get(plus_svg_route))
        .route("/admin/assets/trash.svg", get(trash_svg_route))
        .route("/login", get(login_route))
        .layer(from_fn(logging_middleware))
        .layer(from_fn_with_state(Arc::clone(&cx), force_auth_middleware))
        .with_state(cx)
}
