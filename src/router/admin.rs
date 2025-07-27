use std::sync::Arc;

use super::models::{RouteData, RouteDataIn};
use crate::{
    auth::AnyAuth,
    cx::RouteContext,
    err::AxumResponse,
    router::{
        dash::AdminDashboard,
        request::{DashboardQuery, DeleteRouteAccessData},
        stats::InstanceStats,
    },
    schema::route_data,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Json,
    extract::{Query, State},
    http::header::{CONTENT_TYPE, LOCATION, WWW_AUTHENTICATE},
    response::Response,
};
use axum_auth::AuthBearer;
use axum_extra::TypedHeader;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, delete, insert_into, update};
use diesel_async::RunQueryDsl;

#[axum::debug_handler]
pub async fn set_route_access(
    State(cx): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<RouteDataIn>,
) -> Result<Json<RouteData>, Response> {
    if !cx.validate_master_key(key).await.into_axum()? {
        return Err(anyhow!("Invalid token!")).into_axum();
    }

    let mut db = cx.pool.get().await.unwrap();

    let existing = route_data::table
        .filter(route_data::path.eq(&data.path))
        .select(RouteData::as_select())
        .get_result(&mut db)
        .await;

    Ok(Json(if let Ok(_) = existing {
        update(route_data::table)
            .filter(route_data::path.eq(data.path))
            .set(route_data::visibility.eq(data.visibility))
            .returning(RouteData::as_returning())
            .get_result(&mut db)
            .await
            .into_axum()?
    } else {
        insert_into(route_data::table)
            .values(data)
            .returning(RouteData::as_returning())
            .get_result(&mut db)
            .await
            .into_axum()?
    }))
}

#[axum::debug_handler]
pub async fn delete_route_access(
    State(cx): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
    Json(data): Json<DeleteRouteAccessData>,
) -> Result<Response, Response> {
    if !cx.validate_master_key(key).await.into_axum()? {
        return Err(anyhow!("Invalid token!")).into_axum();
    }

    let mut db = cx.pool.get().await.unwrap();

    delete(route_data::table)
        .filter(route_data::path.eq(data.path))
        .execute(&mut db)
        .await
        .into_axum()?;

    Ok(Response::builder()
        .status(200)
        .body("Success".into())
        .unwrap())
}

#[axum::debug_handler]
pub async fn auth_route(
    State(cx): State<Arc<RouteContext>>,
    auth: Option<TypedHeader<AnyAuth>>,
) -> Response {
    let err = Response::builder()
        .status(401)
        .header(WWW_AUTHENTICATE, "Basic realm=\"Maven\", charset=\"UTF-8\"")
        .header(CONTENT_TYPE, "text/plain")
        .body("Authorizing...".into())
        .unwrap();

    if let Some(auth) = auth {
        if let Ok(_) = auth.get_token(&cx).await {
            Response::builder()
                .status(307)
                .header(LOCATION, "/")
                .header(CONTENT_TYPE, "text/plain")
                .body("Authorized!".into())
                .unwrap()
        } else {
            err
        }
    } else {
        err
    }
}

#[axum::debug_handler]
pub async fn login_route() -> Response {
    Response::builder()
        .status(307)
        .header(LOCATION, "/?force_auth=1")
        .header(CONTENT_TYPE, "text/plain")
        .body("Redirecting to authorizer...".into())
        .unwrap()
}

#[axum::debug_handler]
pub async fn stats_route(
    State(cx): State<Arc<RouteContext>>,
    AuthBearer(key): AuthBearer,
) -> Result<Json<InstanceStats>, Response> {
    if !cx.validate_master_key(key).await.into_axum()? {
        return Err(anyhow!("Invalid token!")).into_axum();
    }

    Ok(Json(cx.stats().await.into_axum()?))
}

#[axum::debug_handler]
pub async fn admin_dashboard_route(
    State(cx): State<Arc<RouteContext>>,
    Query(query): Query<DashboardQuery>,
) -> Result<Response, Response> {
    if !cx.validate_master_key(&query.key).await.into_axum()? {
        return Err(anyhow!("Invalid token!")).into_axum();
    }

    Ok(Response::builder()
        .status(200)
        .body(
            AdminDashboard::get(query.key, &cx)
                .await
                .into_axum()?
                .render()
                .into_axum()?
                .into(),
        )
        .into_axum()?)
}
