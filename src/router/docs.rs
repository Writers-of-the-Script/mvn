use crate::{
    auth::AnyAuth,
    cx::RouteContext,
    err::AxumResponse,
    router::{checks::check_route_access, common::resp_404},
};
use anyhow::{Result, anyhow};
use axum::response::Response;
use axum_extra::TypedHeader;
use std::{
    io::{Cursor, Read},
    sync::Arc,
};
use zip::ZipArchive;

#[derive(Debug)]
pub enum DocType {
    Dokka,
    JavaDoc,
}

impl DocType {
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Dokka => "dokka",
            Self::JavaDoc => "javadoc",
        }
    }
}

pub struct DocsPathParts {
    pub version: String,
    pub artifact: String,
    pub group: String,
    pub path: String,
    pub kind: DocType,
}

fn parse_parts(parts: Vec<String>) -> Result<DocsPathParts> {
    let mut parts = parts
        .into_iter()
        .map(|it| urlencoding::decode(&it).unwrap().to_string())
        .collect::<Vec<_>>();

    let doc_type = parts.remove(0);
    let mut path = Vec::new();

    while parts
        .last()
        .ok_or(anyhow!("Parts was empty before 'raw' was found!"))?
        != "raw"
    {
        let part = parts.pop().unwrap();

        if !part.is_empty() {
            path.push(part);
        }
    }

    path.reverse();

    if path.is_empty() {
        path.push("index.html".into());
    }

    if parts.pop().unwrap() != "raw" {
        return Err(anyhow!("Could not find 'raw' part in parts!"));
    }

    let version = parts.pop().ok_or(anyhow!("Missing component: version"))?;
    let artifact = parts.pop().ok_or(anyhow!("Missing component: artifact"))?;
    let group = parts.join("/");
    let path = path.join("/");

    let kind = match doc_type.as_str() {
        "dokka" => DocType::Dokka,
        "javadoc" => DocType::JavaDoc,
        other => return Err(anyhow!("Unknown doc type: {other}")),
    };

    Ok(DocsPathParts {
        version,
        artifact,
        group,
        path,
        kind,
    })
}

pub async fn docs_handler(
    state: Arc<RouteContext>,
    parts: Vec<String>,
    auth: Option<TypedHeader<AnyAuth>>,
) -> Result<Response, Response> {
    match parse_parts(parts) {
        Ok(DocsPathParts {
            version,
            artifact,
            group,
            path,
            kind,
        }) => {
            let jar_path = format!(
                "/{group}/{artifact}/{version}/{artifact}-{version}-{}.jar",
                kind.suffix()
            );

            let access = check_route_access(&state, &jar_path, &auth)
                .await
                .into_axum()?;

            if !access.read {
                return resp_404();
            }

            let jar_file = state.get_file(&jar_path).await.into_axum()?;
            let jar = jar_file.get_bytes(&state.storage).await.into_axum()?;
            let mut zip = ZipArchive::new(Cursor::new(jar)).into_axum()?;
            let mut target = zip.by_name(&path).into_axum()?;
            let mut buf = Vec::new();

            target.read_to_end(&mut buf).into_axum()?;

            Ok(Response::new(buf.into()))
        }

        Err(_) => resp_404(),
    }
}
