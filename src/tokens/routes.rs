use crate::{auth::AnyAuth, cx::RouteContext, err::AxumResponse};
use anyhow::{Result, anyhow};
use axum::{Json, extract::State, response::Response};
use axum_auth::AuthBearer;
use axum_extra::TypedHeader;

use super::{
    models::{MavenToken, MavenTokenPath},
    request::{AddPathRouteData, AddTokenRouteData},
};

#[axum::debug_handler]
pub async fn new_token_route(
    State(state): State<RouteContext>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<AddTokenRouteData>,
) -> Result<Json<MavenToken>, Response> {
    let lock = state.read();

    if !lock.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid token!")).into_axum()
    } else {
        Ok(Json(lock.create_token(data.into()).await.into_axum()?))
    }
}

#[axum::debug_handler]
pub async fn add_path_route(
    State(state): State<RouteContext>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<AddPathRouteData>,
) -> Result<Json<MavenTokenPath>, Response> {
    let lock = state.read();

    if !lock.validate_master_key(key).await.into_axum()? {
        Err(anyhow!("Invalid token!")).into_axum()
    } else {
        Ok(Json(
            lock.add_token_path(data.token_name, data.path, data.permission)
                .await
                .into_axum()?,
        ))
    }
}

#[axum::debug_handler]
pub async fn get_token_route(
    State(state): State<RouteContext>,
    TypedHeader(auth): TypedHeader<AnyAuth>,
) -> Result<Json<MavenToken>, Response> {
    Ok(Json(auth.get_token(&state.read()).await.into_axum()?))
}
