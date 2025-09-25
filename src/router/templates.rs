use askama::Template;
use humansize::WINDOWS;

use crate::files::models::MavenFile;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub path: String,
    pub folders: Vec<String>,
    pub files: Vec<FileInfo>,
    pub parts: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub type_str: String,
    pub size_str: String,
    pub updated: String,
    pub docs: Option<String>,
}

impl IndexTemplate {
    pub fn parent(path: impl AsRef<str>) -> String {
        let mut parts = path
            .as_ref()
            .trim_end_matches('/')
            .split("/")
            .collect::<Vec<_>>();

        parts.pop();

        let path = parts.join("/");

        if path == "" { "/".into() } else { path }
    }
}

impl FileInfo {
    pub fn hash(
        name: impl AsRef<str>,
        updated: impl AsRef<str>,
        size: impl AsRef<str>,
    ) -> FileInfo {
        let name = name.as_ref().into();
        let updated = updated.as_ref().into();

        Self {
            name,
            updated,
            size_str: size.as_ref().into(),
            type_str: "Text File".into(),
            docs: None,
        }
    }

    pub fn new(path: impl AsRef<str>, file: &MavenFile, size: u64) -> FileInfo {
        let path = path.as_ref();
        let updated = file.uploaded.format("%Y-%m-%d %I:%M %p UTC").to_string();

        if path.ends_with(".md5")
            || path.ends_with(".sha1")
            || path.ends_with(".sha256")
            || path.ends_with(".sha512")
        {
            return FileInfo::hash(
                path.split("/").last().unwrap(),
                updated,
                humansize::format_size(size, WINDOWS.decimal_places(2)),
            );
        }

        let name = file.path.split("/").last().unwrap().to_string();
        let size_str = humansize::format_size(file.size as usize, WINDOWS.decimal_places(2));

        FileInfo {
            name,
            updated,
            size_str,
            type_str: file.kind.clone(),
            docs: if path.ends_with("-javadoc.jar") {
                Some("javadoc".into())
            } else if path.ends_with("-dokka.jar") {
                Some("dokka".into())
            } else {
                None
            },
        }
    }
}
