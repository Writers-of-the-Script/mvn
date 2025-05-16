use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;
use diesel::sqlite::Sqlite;
use random_string::charsets::ALPHANUMERIC;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(Sqlite))]
pub struct MavenFile {
    pub id: i32,
    pub path: String,
    pub parent: String,
    pub size: i64,
    pub uploaded: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::deleted_files)]
#[diesel(check_for_backend(Sqlite))]
pub struct DeletedMavenFile {
    pub id: i32,
    pub path: String,
    pub parent: String,
    pub size: i64,
    pub uploaded: NaiveDateTime,
    pub deleted: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::tokens)]
#[diesel(check_for_backend(Sqlite))]
pub struct MavenToken {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::master_keys)]
#[diesel(check_for_backend(Sqlite))]
pub struct MasterKey {
    pub id: i32,
    pub value: String,
    pub created: NaiveDateTime,
    pub is_init: bool,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Insertable,
    Identifiable,
    Queryable,
    Selectable,
    Associations,
)]
#[diesel(table_name = crate::schema::token_paths)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(belongs_to(MavenToken, foreign_key = token))]
pub struct MavenTokenPath {
    pub id: i32,
    pub token: i32,
    pub path: String,
    pub added: NaiveDateTime,
    pub permission: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(Sqlite))]
pub struct MavenFileIn {
    pub path: String,
    pub size: i64,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::deleted_files)]
#[diesel(check_for_backend(Sqlite))]
pub struct DeletedMavenFileIn {
    pub path: String,
    pub size: i64,
    pub deleted: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::tokens)]
#[diesel(check_for_backend(Sqlite))]
pub struct MavenTokenIn {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Associations)]
#[diesel(table_name = crate::schema::token_paths)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(belongs_to(MavenToken, foreign_key = token))]
pub struct MavenTokenPathIn {
    pub token: i32,
    pub path: String,
    pub permission: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::master_keys)]
#[diesel(check_for_backend(Sqlite))]
pub struct MasterKeyIn {
    pub value: String,
    pub is_init: bool,
}

impl MavenTokenPath {
    pub fn can_read(&self) -> bool {
        self.permission == 0 || self.is_read_write()
    }

    pub fn can_write(&self) -> bool {
        self.permission == 1 || self.is_read_write()
    }

    pub fn is_read_write(&self) -> bool {
        self.permission == 2
    }
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
        Self {
            name: name.as_ref().into(),
            value: random_string::generate(32, ALPHANUMERIC),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum MavenTokenPermissions {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

impl MavenTokenPermissions {
    pub fn value(&self) -> i16 {
        match self {
            Self::Read => 0,
            Self::Write => 1,
            Self::ReadWrite => 2,
        }
    }

    pub fn from_value(value: i16) -> Result<Self> {
        match value {
            0 => Ok(Self::Read),
            1 => Ok(Self::Write),
            2 => Ok(Self::ReadWrite),
            _ => Err(anyhow!("Unknown value: {value}")),
        }
    }
}
