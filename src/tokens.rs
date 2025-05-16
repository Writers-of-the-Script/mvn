use crate::{
    auth::AnyAuth,
    cx::{RouteContext, RouteContextInner},
    err::AxumResponse,
    models::{MavenToken, MavenTokenIn, MavenTokenPath, MavenTokenPermissions},
};
use anyhow::{Result, anyhow};
use axum::{Json, extract::State, response::Response};
use axum_auth::AuthBearer;
use axum_extra::TypedHeader;
use parking_lot::RwLockReadGuard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTokenRouteData {
    pub name: String,
    pub value: Option<String>,
}

impl Into<MavenTokenIn> for AddTokenRouteData {
    fn into(self) -> MavenTokenIn {
        match self.value {
            Some(value) => MavenTokenIn {
                name: self.name,
                value,
            },

            None => MavenTokenIn::new_random(self.name),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPathRouteData {
    pub token_name: String,
    pub path: String,
    pub permission: MavenTokenPermissions,
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

impl MavenToken {
    pub async fn can_write_to<'a>(
        &self,
        cx: &RwLockReadGuard<'a, RouteContextInner>,
        path: impl AsRef<str>,
    ) -> Result<bool> {
        let path = path.as_ref();
        let paths = cx.get_token_writable_paths(&self).await?;

        Ok(paths.iter().any(|it| path.starts_with(it)))
    }

    pub async fn can_read_from<'a>(
        &self,
        cx: &RwLockReadGuard<'a, RouteContextInner>,
        path: impl AsRef<str>,
    ) -> Result<bool> {
        let path = path.as_ref();
        let paths = cx.get_token_readable_paths(&self).await?;

        Ok(paths.iter().any(|it| path.starts_with(it)))
    }
}
