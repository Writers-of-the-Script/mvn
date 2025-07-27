use std::sync::Arc;

use super::{
    checks::{AccessChecker, check_route_access},
    common::resp_404,
    templates::{FileInfo, IndexTemplate},
};
use crate::{auth::AnyAuth, cx::RouteContext, err::AxumResponse};
use anyhow::Result;
use askama::Template;
use axum::{http::header::CONTENT_TYPE, response::Response};
use axum_extra::TypedHeader;

pub async fn get_handler(
    state: Arc<RouteContext>,
    path: impl AsRef<str>,
    auth: Option<TypedHeader<AnyAuth>>,
) -> Result<Response, Response> {
    debug!("Checking route access...");

    let path = path.as_ref();
    let access = check_route_access(&state, path, &auth).await.into_axum()?;

    match state.get_file_for_route(&path).await {
        Ok(it) => {
            debug!("Found file!");

            if !access.read {
                debug!("No access!");

                return resp_404();
            }

            debug!("Getting file content...");

            let bytes = it.get_content(&path, &state.storage).await.into_axum()?;

            debug!("Responding...");

            Ok(Response::builder()
                .status(200)
                .header(
                    CONTENT_TYPE,
                    if path.ends_with(".md5")
                        || path.ends_with(".sha1")
                        || path.ends_with(".sha256")
                        || path.ends_with(".sha512")
                    {
                        "text/plain"
                    } else {
                        match infer::get(&bytes)
                            .map(|it| it.mime_type())
                            .unwrap_or_else(|| {
                                if bytes.is_ascii() {
                                    "text/plain"
                                } else {
                                    "application/octet-stream"
                                }
                            }) {
                            "application/zip" => {
                                if path.ends_with(".jar") {
                                    "application/java-archive"
                                } else {
                                    "application/zip"
                                }
                            }
                            other => other,
                        }
                        .into()
                    },
                )
                .body(bytes.into())
                .into_axum()?)
        }

        Err(_) => match state.index().get(&format!("{}/", path).replace("//", "/")) {
            Some(entries) => {
                debug!("Found folder!");

                if !access.index {
                    debug!("No access!");

                    return resp_404();
                }

                debug!("Building index...");

                let mut folders = Vec::new();
                let mut files = Vec::new();

                let checker = AccessChecker::new(&state, &auth).await.into_axum()?;

                for item in entries.iter() {
                    debug!("Checking: {item}");

                    let acc = checker.check(item);

                    if item.ends_with("/") {
                        if !acc.index {
                            continue;
                        }

                        debug!("Is folder - OK");

                        folders.push(
                            item.trim_start_matches(&path)
                                .trim_start_matches('/')
                                .trim_end_matches('/')
                                .into(),
                        );
                    } else {
                        if !acc.read {
                            continue;
                        }

                        debug!("Is file - OK");

                        let file_path = format!("{}/{}", path, item).replace("//", "/");

                        let Ok(file) = state.get_file_for_route(&file_path).await else {
                            continue;
                        };

                        files.push(FileInfo::new(
                            &file_path,
                            &file,
                            file.get_size(&file_path, &state.storage)
                                .await
                                .into_axum()?,
                        ));
                    }
                }

                debug!("Sorting index...");

                folders.sort();
                files.sort_by_key(|f| f.name.clone());

                debug!("Building template...");

                let data = IndexTemplate {
                    path: format!("{}/", path).replace("//", "/"),
                    files,
                    folders,
                    title: "The Broken Script Maven".into(),
                };

                debug!("Responding...");

                Ok(Response::builder()
                    .status(200)
                    .body(data.render().into_axum()?.into())
                    .into_axum()?)
            }

            None => resp_404(),
        },
    }
}
