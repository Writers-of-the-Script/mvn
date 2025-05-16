use crate::cx::RouteContextInner;
use anyhow::Result;
use std::collections::HashMap;

impl RouteContextInner {
    pub async fn index_dirs(&mut self) -> Result<()> {
        self.dirs = vec!["/".into()];
        self.dir_entries = HashMap::new();
        self.dir_entries.insert("/".into(), Vec::new());

        for (key, file) in &self.get_all_files().await? {
            let mut parts = key.split("/").map(|v| v.to_string()).collect::<Vec<_>>();

            parts.pop();

            let mut current = Vec::new();

            for part in &parts {
                current.push(part.clone());

                let cur = format!("/{}/", current.join("/")).replace("//", "/");

                if !self.dirs.contains(&cur) {
                    self.dirs.push(cur.clone());
                }

                if !self.dir_entries.contains_key(&cur) {
                    self.dir_entries.insert(cur.clone(), Vec::new());
                }

                let mut parent_list = current.clone();

                parent_list.pop();

                let parent = format!("/{}/", parent_list.join("/")).replace("//", "/");
                let entries = self.dir_entries.get_mut(&parent).unwrap();

                if !entries.contains(&cur) && cur != "/" {
                    entries.push(cur);
                }
            }

            let parent = format!("/{}/", parts.join("/")).replace("//", "/");
            let entries = self.dir_entries.get_mut(&parent).unwrap();

            for route in file.routes() {
                let route = route.split("/").last().unwrap().into();

                if !entries.contains(&route) {
                    entries.push(route);
                }
            }
        }

        Ok(())
    }

    pub fn get_path(&self, route: impl AsRef<str>) -> String {
        format!(
            "/{}",
            route
                .as_ref()
                .trim_end_matches(".md5")
                .trim_end_matches(".sha1")
                .trim_end_matches(".sha256")
                .trim_end_matches(".sha512")
        )
        .replace("//", "/")
    }
}
