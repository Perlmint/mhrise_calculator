use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Skill {
    pub id: String,
    pub names: HashMap<String, String>,
    pub texts: HashMap<String, String>,
}
