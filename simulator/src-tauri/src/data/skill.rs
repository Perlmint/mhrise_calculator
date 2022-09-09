use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub id: String,

    #[serde(rename = "maxLevel")]
    pub max_level: i32,

    pub names: HashMap<String, String>,
    pub texts: HashMap<String, String>,
}
