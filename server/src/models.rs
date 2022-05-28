use super::schema::players;

#[derive(Debug, Clone, Queryable, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub itsf_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub dtfb_license: Option<String>,
    pub birth_year: i32,
    pub country_code: Option<String>,
    pub category: Option<String>,
}

#[derive(Insertable)]
#[table_name = "players"]
pub struct NewPlayer<'a> {
    pub itsf_id: i32,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub dtfb_license: Option<&'a str>,
    pub birth_year: i32,
    pub country_code: Option<&'a str>,
    pub category: Option<&'a str>,
}
