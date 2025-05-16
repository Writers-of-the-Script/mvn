use crate::{
    auth::AnyAuth,
    cx::RouteContext,
    docs::docs_handler,
    err::AxumResponse,
    templates::{FileInfo, IndexTemplate},
};
use anyhow::{Result, anyhow};
use askama::Template;
use axum::{
    extract::{Request, State},
    http::{Method, header::CONTENT_TYPE},
    response::Response,
};
use axum_extra::TypedHeader;
use http_body_util::BodyExt;
use minify_html_onepass::{Cfg, copy};

async fn get_handler(state: RouteContext, path: String) -> Result<Response, Response> {
    let read = state.read();

    match read.get_file_for_route(&path).await {
        Ok(it) => {
            let bytes = it.get_content(&path, &read.storage).await.into_axum()?;

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

        Err(_) => match read
            .dir_entries
            .get(&format!("{}/", path).replace("//", "/"))
        {
            Some(entries) => {
                let mut folders = Vec::new();
                let mut files = Vec::new();

                for item in entries {
                    if item.ends_with("/") {
                        folders.push(
                            item.trim_start_matches(&path)
                                .trim_start_matches('/')
                                .trim_end_matches('/')
                                .into(),
                        );
                    } else {
                        let file_path = format!("{}/{}", path, item).replace("//", "/");

                        let Ok(file) = read.get_file_for_route(&file_path).await else {
                            continue;
                        };

                        files.push(FileInfo::new(
                            &file_path,
                            &file,
                            file.get_size(&file_path, &read.storage).await.into_axum()?,
                        ));
                    }
                }

                folders.sort();
                files.sort_by_key(|f| f.name.clone());

                let data = IndexTemplate {
                    path: format!("{}/", path).replace("//", "/"),
                    files,
                    folders,
                    title: "Redstone's Maven".into(),
                };

                let html = copy(
                    data.render().into_axum()?.as_bytes(),
                    &Cfg {
                        minify_css: true,
                        minify_js: true,
                    },
                )
                .into_axum()?;

                Ok(Response::builder()
                    .status(200)
                    .body(html.into())
                    .into_axum()?)
            }

            None => Err(Response::builder()
                .status(404)
                .body("404 Not Found".into())
                .into_axum()?),
        },
    }
}

#[axum::debug_handler]
pub async fn route_handler(
    State(state): State<RouteContext>,
    auth: Option<TypedHeader<AnyAuth>>,
    req: Request,
) -> Result<Response, Response> {
    let path = req.uri().path().to_string();

    if path.starts_with("/docs/") {
        let (_, parts) = path.split_once("/docs/").unwrap();

        let parts = parts
            .split("/")
            .map(|it| it.to_string())
            .collect::<Vec<_>>();

        return docs_handler(state, parts).await;
    }

    match *req.method() {
        Method::GET => get_handler(state, path).await,

        Method::PUT => {
            {
                let read = state.read();

                let token = auth
                    .ok_or(anyhow!("Auth is required!"))
                    .into_axum()?
                    .get_token(&read)
                    .await
                    .into_axum()?;

                if !token.can_write_to(&read, &path).await.into_axum()? {
                    return Err(Response::builder()
                        .status(403)
                        .body("403 Access Denied".into())
                        .into_axum()?);
                }
            }

            {
                let body = req.into_body();
                let collected = body.collect().await.into_axum()?;
                let lock = state.read();

                lock.upload(&path, collected.to_bytes()).await.into_axum()?;
            }

            {
                let mut lock = state.write();

                lock.index_dirs().await.into_axum()?;
            }

            get_handler(state, path).await
        }

        Method::DELETE => {
            {
                let read = state.read();

                let token = auth
                    .ok_or(anyhow!("Auth is required!"))
                    .into_axum()?
                    .get_token(&read)
                    .await
                    .into_axum()?;

                if !token.can_write_to(&read, &path).await.into_axum()? {
                    return Err(Response::builder()
                        .status(403)
                        .body("403 Access Denied".into())
                        .into_axum()?);
                }
            }

            let mut write = state.write();

            if write.has_file(&path).await.into_axum()? {
                write.delete_file(&path).await.into_axum()?;
                write.index_dirs().await.into_axum()?;
            }

            Ok(Response::builder()
                .status(200)
                .body("200 Ok".into())
                .into_axum()?)
        }

        _ => Err(Response::builder()
            .status(405)
            .body("405 Method Not Allowed".into())
            .into_axum()?),
    }
}
