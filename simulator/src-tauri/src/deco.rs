use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Decoration {
    id: String,
    names: HashMap<String, String>,
    #[serde(rename = "skillNames")]
    skill_names: HashMap<String, String>,
    texts: HashMap<String, String>,
    #[serde(rename = "skillId")]
    skill_id: String,
    #[serde(rename = "skillLevel")]
    skill_level: i32,
    #[serde(rename = "slotSize")]
    slot_size: i32,
}
