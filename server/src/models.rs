#[derive(Debug, Clone, Queryable, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub itsf_license: String,
    pub dtfb_license: Option<String>,
    pub birth_year: i32,
    pub country_code: Option<String>,
    pub category: Option<String>,
}
