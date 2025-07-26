use std::sync::Arc;

use super::request::{AddPathRouteData, AddTokenRouteData};
use crate::{
    auth::AnyAuth,
    cx::RouteContext,
    err::AxumResponse,
    router::request::{RemovePathRouteData, TokenInfoRouteData},
    tokens::models::{MavenTokenPath, MavenTokenSafe},
};
use anyhow::{Result, anyhow};
use axum::{Json, extract::State, response::Response};
use axum_auth::AuthBearer;
use axum_extra::TypedHeader;

#[axum::debug_handler]
pub async fn new_token_route(
    State(state): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<AddTokenRouteData>,
) -> Result<Json<MavenTokenSafe>, Response> {
    let generated = data.value.is_none();

    if !state.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid master key!")).into_axum()
    } else {
        Ok(Json(
            state
                .create_token(data.into(), generated)
                .await
                .into_axum()?,
        ))
    }
}

#[axum::debug_handler]
pub async fn delete_token_route(
    State(state): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<TokenInfoRouteData>,
) -> Result<Response, Response> {
    if !state.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid master key!")).into_axum()
    } else {
        state.delete_token(data.name).await.into_axum()?;

        Ok(Response::builder()
            .status(200)
            .body("Success".into())
            .unwrap())
    }
}

#[axum::debug_handler]
pub async fn get_token_paths_route(
    State(state): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<TokenInfoRouteData>,
) -> Result<Json<Vec<MavenTokenPath>>, Response> {
    if !state.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid master key!")).into_axum()
    } else {
        Ok(Json(
            state.get_token_paths_by_name(data.name).await.into_axum()?,
        ))
    }
}

#[axum::debug_handler]
pub async fn add_path_route(
    State(state): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<AddPathRouteData>,
) -> Result<Json<MavenTokenPath>, Response> {
    if !state.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid token!")).into_axum()
    } else {
        Ok(Json(
            state
                .add_token_path(data.token_name, data.path, data.permission)
                .await
                .into_axum()?,
        ))
    }
}

#[axum::debug_handler]
pub async fn delete_path_route(
    State(state): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<RemovePathRouteData>,
) -> Result<Response, Response> {
    if !state.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid master key!")).into_axum()
    } else {
        state
            .remove_token_path(data.token_name, data.path)
            .await
            .into_axum()?;

        Ok(Response::builder()
            .status(200)
            .body("Success".into())
            .unwrap())
    }
}

#[axum::debug_handler]
pub async fn get_token_route(
    State(state): State<Arc<RouteContext>>,
    TypedHeader(auth): TypedHeader<AnyAuth>,
) -> Result<Json<MavenTokenSafe>, Response> {
    Ok(Json(auth.get_token(&state).await.into_axum()?.safe(None)))
}
