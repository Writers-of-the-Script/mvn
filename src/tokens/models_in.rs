use crate::tokens::models::MavenToken;
use random_string::charsets::ALPHANUMERIC;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::tokens)]
pub struct MavenTokenIn {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Associations)]
#[diesel(table_name = crate::schema::token_paths)]
#[diesel(belongs_to(MavenToken, foreign_key = token))]
pub struct MavenTokenPathIn {
    pub token: i32,
    pub path: String,
    pub permission: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::master_keys)]
pub struct MasterKeyIn {
    pub value: String,
    pub is_init: bool,
}

impl MavenTokenPathIn {
    pub fn read(token: i32, path: impl AsRef<str>) -> Self {
        Self::new(token, path, 0)
    }

    pub fn write(token: i32, path: impl AsRef<str>) -> Self {
        Self::new(token, path, 1)
    }

    pub fn read_write(token: i32, path: impl AsRef<str>) -> Self {
        Self::new(token, path, 2)
    }

    pub fn new(token: i32, path: impl AsRef<str>, permission: i16) -> Self {
        Self {
            token,
            path: path.as_ref().into(),
            permission,
        }
    }
}

impl MavenTokenIn {
    pub fn new_random(name: impl AsRef<str>) -> Self {
        debug!("Generating new random token...");
        
        Self {
            name: name.as_ref().into(),
            value: random_string::generate(32, ALPHANUMERIC),
        }
    }
}
