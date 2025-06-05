use crate::tokens::{models_in::MavenTokenIn, perms::MavenTokenPermissions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTokenRouteData {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPathRouteData {
    pub token_name: String,
    pub path: String,
    pub permission: MavenTokenPermissions,
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
