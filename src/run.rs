use anyhow::Result;
use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post, put},
};
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    assets::{jbm_font_route, page_js_route, robots_txt_route},
    cx::RouteContextInner,
    db::{connect, migrate},
    logging::logging_middleware,
    router::route_handler,
    tokens::{
        master::create_default_key,
        routes::{add_path_route, get_token_route, new_token_route},
    },
};

pub async fn run() -> Result<()> {
    info!("Connecting to the database...");

    let pool = connect(None)?;

    info!("Running migrations...");

    migrate(&pool).await?;
    create_default_key(&pool).await?;

    info!("Building context...");

    let cx = RouteContextInner::create(None, pool).await?;

    info!("Creating app...");

    let app = Router::new()
        .fallback(route_handler)
        .route("/api/token", put(new_token_route))
        .route("/api/token", post(add_path_route))
        .route("/api/token", get(get_token_route))
        .route("/assets/fonts/jetbrains-mono.woff2", get(jbm_font_route))
        .route("/assets/js/page.js", get(page_js_route))
        .route("/robots.txt", get(robots_txt_route))
        .layer(from_fn(logging_middleware))
        .with_state(cx);

    info!("Binding listener...");

    let listener = TcpListener::bind("0.0.0.0:4000").await?;

    info!("Service started on http://0.0.0.0:4000");

    axum::serve(listener, app).await?;

    Ok(())
}
