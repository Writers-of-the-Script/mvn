#![feature(let_chains)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde;

pub mod assets;
pub mod auth;
pub mod cx;
pub mod db;
pub mod docs;
pub mod err;
pub mod files;
pub mod hashes;
pub mod logging;
pub mod models;
pub mod router;
pub mod schema;
pub mod swc;
pub mod templates;
pub mod tokens;
pub mod types;

use anyhow::Result;
use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post, put},
};
use cx::RouteContextInner;
use db::{connect, migrate};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into};
use diesel_async::RunQueryDsl;
use logging::logging_middleware;
use models::{MasterKey, MasterKeyIn};
use random_string::charsets::ALPHANUMERIC;
use router::route_handler;
use schema::master_keys;
use tokio::net::TcpListener;
use tracing::info;

pub async fn run() -> Result<()> {
    info!("Connecting to the database...");

    let pool = connect(None)?;

    info!("Running migrations...");

    migrate(&pool).await?;

    {
        let mut conn = pool.get().await?;

        if master_keys::table
            .filter(master_keys::is_init.eq(true))
            .select(MasterKey::as_select())
            .get_result(&mut conn)
            .await
            .is_err()
        {
            info!("Creating new master key...");

            let new = insert_into(master_keys::table)
                .values(MasterKeyIn {
                    value: random_string::generate(32, ALPHANUMERIC),
                    is_init: true,
                })
                .returning(MasterKey::as_returning())
                .get_result(&mut conn)
                .await?;

            info!(
                ">> Your master key is {}. Write it down, it won't be displayed again!",
                new.value
            );
        }
    }

    info!("Building context...");

    let cx = RouteContextInner::create(None, pool).await?;

    info!("Creating app...");

    let app = Router::new()
        .fallback(route_handler)
        .route("/api/token", put(tokens::new_token_route))
        .route("/api/token", post(tokens::add_path_route))
        .route("/api/token", get(tokens::get_token_route))
        .route(
            "/assets/fonts/jetbrains-mono.woff2",
            get(assets::jbm_font_route),
        )
        .route("/assets/js/page.js", get(assets::page_js_route))
        .route("/robots.txt", get(assets::robots_txt_route))
        .layer(from_fn(logging_middleware))
        .with_state(cx);

    info!("Binding listener...");

    let listener = TcpListener::bind("0.0.0.0:4000").await?;

    info!("Service started on http://0.0.0.0:4000");

    axum::serve(listener, app).await?;

    Ok(())
}
