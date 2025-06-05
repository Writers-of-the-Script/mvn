use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::route_data)]
pub struct RouteData {
    pub id: i32,
    pub path: String,
    pub visibility: i16,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::route_data)]
pub struct RouteDataIn {
    pub path: String,
    pub visibility: i16,
}

impl RouteData {
    pub fn is_public(&self) -> bool {
        self.visibility == 0
    }

    pub fn is_hidden(&self) -> bool {
        self.visibility == 1
    }

    pub fn is_private(&self) -> bool {
        self.visibility == 2
    }
}
