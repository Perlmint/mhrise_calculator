use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Decoration {
    pub id: String,
    pub names: HashMap<String, String>,

    #[serde(rename = "skillNames")]
    pub skill_names: HashMap<String, String>,
    // pub texts: HashMap<String, String>,
    #[serde(rename = "skillId")]
    pub skill_id: String,

    #[serde(rename = "skillLevel")]
    pub skill_level: i32,

    #[serde(rename = "slotSize")]
    pub slot_size: i32,
}
