use serde::{Deserialize, Serialize};
use shared::GoatParams;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Goat {
    id: Option<i64>,
    pub params: GoatParams,
}

#[derive(Deserialize)]
pub struct NamePayload {
    pub name: String,
}
