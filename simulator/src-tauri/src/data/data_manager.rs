use std::collections::HashMap;

use super::armor::{AnomalyArmor, BaseArmor, Talisman};
use super::deco::Decoration;
use super::skill::Skill;

#[derive(Default)]
pub struct DataManager {
    pub armors: HashMap<String, BaseArmor>,
    pub skills: HashMap<String, Skill>,
    pub decos: HashMap<String, Decoration>,

    pub anomaly_armors: Vec<AnomalyArmor>,

    pub talismans: Vec<Talisman>,
}
