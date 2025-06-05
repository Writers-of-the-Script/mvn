use super::models::RouteData;
use crate::{auth::AnyAuth, cx::RouteContext, schema::route_data};
use anyhow::Result;
use axum_extra::TypedHeader;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RouteAccessInfo {
    /// Can the user download files?
    pub read: bool,

    /// Can the user view the list of files?
    pub index: bool,
}

impl RouteAccessInfo {
    pub fn read_only() -> Self {
        Self {
            read: true,
            index: false,
        }
    }

    pub fn index_only() -> Self {
        Self {
            read: false,
            index: true,
        }
    }

    pub fn read_index() -> Self {
        Self {
            read: true,
            index: true,
        }
    }

    pub fn none() -> Self {
        Self {
            read: false,
            index: false,
        }
    }
}

pub async fn check_route_access(
    cx: &RouteContext,
    route: impl AsRef<str>,
    auth: &Option<TypedHeader<AnyAuth>>,
) -> Result<RouteAccessInfo> {
    let mut db = cx.pool.get().await?;
    let route_str = route.as_ref();

    let routes = route_data::table
        .select(RouteData::as_select())
        .load(&mut db)
        .await?;

    let Some(route) = routes
        .into_iter()
        .filter(|it| route_str.starts_with(&it.path))
        .sorted_by_key(|it| it.path.len())
        .last()
    else {
        return Ok(RouteAccessInfo::read_index());
    };

    if route.is_public() {
        Ok(RouteAccessInfo::read_index())
    } else {
        match auth {
            Some(header) => {
                let token = header.get_token(cx).await?;

                if token.can_read_from(cx, &route_str).await? {
                    Ok(RouteAccessInfo::read_index())
                } else {
                    Ok(if route.is_hidden() {
                        RouteAccessInfo::read_only()
                    } else {
                        RouteAccessInfo::none()
                    })
                }
            }

            None => Ok(if route.is_hidden() {
                RouteAccessInfo::read_only()
            } else {
                RouteAccessInfo::none()
            }),
        }
    }
}

pub struct AccessChecker {
    routes: Vec<RouteData>,
    paths: Option<Vec<String>>,
}

impl AccessChecker {
    pub async fn new(cx: &RouteContext, auth: &Option<TypedHeader<AnyAuth>>) -> Result<Self> {
        let mut db = cx.pool.get().await?;

        let routes = route_data::table
            .select(RouteData::as_select())
            .load(&mut db)
            .await?;

        let paths = match auth {
            Some(header) => Some(
                cx.get_token_readable_paths(&header.get_token(cx).await?)
                    .await?,
            ),
            None => None,
        };

        Ok(Self { paths, routes })
    }

    pub fn check(&self, route: impl AsRef<str>) -> RouteAccessInfo {
        let route_str = route.as_ref();

        let Some(route) = &self
            .routes
            .iter()
            .filter(|it| route_str.starts_with(&it.path))
            .sorted_by_key(|it| it.path.len())
            .last()
        else {
            return RouteAccessInfo::read_index();
        };

        if route.is_public() {
            RouteAccessInfo::read_index()
        } else {
            match &self.paths {
                Some(paths) => {
                    if paths.iter().any(|it| route_str.starts_with(it)) {
                        RouteAccessInfo::read_index()
                    } else {
                        if route.is_hidden() {
                            RouteAccessInfo::read_only()
                        } else {
                            RouteAccessInfo::none()
                        }
                    }
                }

                None => {
                    if route.is_hidden() {
                        RouteAccessInfo::read_only()
                    } else {
                        RouteAccessInfo::none()
                    }
                }
            }
        }
    }
}
