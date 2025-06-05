use crate::cx::RouteContext;
use anyhow::Result;
use chashmap::CHashMap;
use diesel::pg::Pg;
use diesel_async::AsyncConnection;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static DIR_ENTRIES: Lazy<CHashMap<String, Vec<String>>> = Lazy::new(CHashMap::new);

impl RouteContext {
    pub fn index(&self) -> &CHashMap<String, Vec<String>> {
        &*DIR_ENTRIES
    }

    pub async fn index_dirs(&self, conn: &mut impl AsyncConnection<Backend = Pg>) -> Result<()> {
        debug!("Resetting index...");

        let mut map = HashMap::new();

        map.insert("/".into(), Vec::new());

        debug!("Fetching files from database...");

        for (key, file) in &self.get_all_files_inner(conn).await? {
            debug!("Process: {key}");

            let mut parts = key.split("/").map(|v| v.to_string()).collect::<Vec<_>>();

            parts.pop();

            let mut current = Vec::new();

            for part in &parts {
                current.push(part.clone());

                let cur = format!("/{}/", current.join("/")).replace("//", "/");

                debug!("Inserting if needed: {cur}");

                if !map.contains_key(&cur) {
                    map.insert(cur.clone(), Vec::new());
                }

                let mut parent_list = current.clone();

                parent_list.pop();

                let parent = format!("/{}/", parent_list.join("/")).replace("//", "/");
                let entries = map.get_mut(&parent).unwrap();

                if !entries.contains(&cur) && cur != "/" {
                    entries.push(cur);
                }
            }

            let parent = format!("/{}/", parts.join("/")).replace("//", "/");

            debug!("Inserting hash routes for {parent}...");

            let entries = map.get_mut(&parent).unwrap();

            for route in file.routes() {
                let route = route.split("/").last().unwrap().into();

                if !entries.contains(&route) {
                    entries.push(route);
                }
            }
        }

        DIR_ENTRIES.clear();

        for (k, v) in map {
            DIR_ENTRIES.insert_new(k, v);
        }

        Ok(())
    }
}
