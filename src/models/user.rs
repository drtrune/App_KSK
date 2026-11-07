use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RoleRecord {
    pub code: String,
    pub name: String,
    pub description: String,
    pub is_system: bool,
    pub permissions: Vec<String>,
}

