use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum Breed {
    Beetal,
    Jamunapari,
    Barbari,
    Sirohi,
    Osmanabadi,
    BlackBengal,
    Kutchi,
    Kaghani,
    Chegu,
    Jakhrana,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Vaccine {
    Rabies,
    Cdt,
    Clostridium,
    FootAndMouth,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Disease {
    FootRot,
    Mastitis,
    Parasites,
    Pneumonia,
    Other(String),
}

// VaccineRf and DiseaseRef currently look the same.
// However, we can add more functionality like booster date for vaccine
// and symptoms for disease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaccineRef {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseRef {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Goat {
    pub id: Option<i64>,
    pub breed: Breed,
    pub name: String,
    pub gender: Gender,
    pub offspring: i32,
    pub cost: f64,
    pub weight: f64,
    pub current_price: f64,
    pub diet: String,
    pub last_bred: Option<String>,
    pub health_status: String,
    pub vaccinations: Vec<VaccineRef>,
    pub diseases: Vec<DiseaseRef>,
}

#[derive(Deserialize)]
pub struct IdPayload {
    pub id: i64,
}
