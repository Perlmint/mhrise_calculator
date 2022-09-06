use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Skill {
    id: String,
    names: HashMap<String, String>,
    texts: HashMap<String, String>,
}
