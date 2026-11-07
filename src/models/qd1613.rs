use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QdSpecialty {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QdDisease {
    pub id: String,
    pub name: String,
    pub spec_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QdClassification {
    pub id: String,
    pub disease_id: String,
    pub name: String,
    pub score: Option<i32>,
    pub ref_rule: Option<String>,
    pub ref_disease_id: Option<String>,
    pub modifier: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Qd1613CatalogData {
    pub specialties: Vec<QdSpecialty>,
    pub diseases: Vec<QdDisease>,
    pub classifications: Vec<QdClassification>,
}

