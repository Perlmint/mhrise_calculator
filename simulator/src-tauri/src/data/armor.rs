use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArmorPart {
    #[serde(rename = "helm")]
    Helm,
    #[serde(rename = "torso")]
    Torso,
    #[serde(rename = "arm")]
    Arm,
    #[serde(rename = "waist")]
    Waist,
    #[serde(rename = "feet")]
    Feet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SexType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "female")]
    Female,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArmorStat {
    pub defense: i32,
    #[serde(rename = "fireRes")]
    pub fire_res: i32,
    #[serde(rename = "waterRes")]
    pub water_res: i32,
    #[serde(rename = "iceRes")]
    pub ice_res: i32,
    #[serde(rename = "elecRes")]
    pub elec_res: i32,
    #[serde(rename = "dragonRes")]
    pub dragon_res: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArmorSkill {
    pub id: String,
    pub level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseArmor {
    pub id: String,
    pub part: ArmorPart,

    #[serde(rename = "sexType")]
    pub sex_type: SexType,

    pub names: HashMap<String, String>,
    pub rarity: i32,
    pub stat: ArmorStat,
    pub skills: Vec<ArmorSkill>,
}

#[derive(Serialize, Clone)]
pub struct AnomalyArmor {
    pub original: BaseArmor,

    #[serde(rename = "statDiff")]
    pub stat_diff: ArmorStat,

    #[serde(rename = "slotDiffs")]
    pub slot_diffs: Vec<i32>,

    #[serde(rename = "skillDiffs")]
    pub skill_diffs: Vec<ArmorSkill>,
}

pub struct TalismanSkill {
    pub id: String,
    pub level: i32,
}

pub struct Talisman {
    pub skills: Vec<TalismanSkill>,
    pub slot_sizes: Vec<i32>,
}
