use chrono::NaiveDateTime;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Insertable,
    Identifiable,
    Queryable,
    Selectable,
)]
#[diesel(table_name = crate::schema::tokens)]
pub struct MavenToken {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenTokenSafe {
    pub id: i32,
    pub name: String,
    pub created: NaiveDateTime,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::master_keys)]
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
#[diesel(belongs_to(MavenToken, foreign_key = token))]
pub struct MavenTokenPath {
    pub id: i32,
    pub token: i32,
    pub path: String,
    pub added: NaiveDateTime,
    pub permission: i16,
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

impl MavenToken {
    pub fn safe(self, value: Option<String>) -> MavenTokenSafe {
        MavenTokenSafe {
            id: self.id,
            name: self.name,
            created: self.created,
            value,
        }
    }
}
