use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
enum ArmorPart {
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

#[derive(Deserialize, Debug)]
enum SexType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "female")]
    Female,
}

#[derive(Deserialize, Debug)]
struct ArmorStat {
    defense: i32,
    #[serde(rename = "fireRes")]
    fire_res: i32,
    #[serde(rename = "waterRes")]
    water_res: i32,
    #[serde(rename = "iceRes")]
    ice_res: i32,
    #[serde(rename = "elecRes")]
    elec_res: i32,
    #[serde(rename = "dragonRes")]
    dragon_res: i32,
}

#[derive(Deserialize, Debug)]
struct ArmorSkill {
    name: String,
    level: i32,
}

#[derive(Deserialize, Debug)]
pub struct AnomalyArmor {
    id: String,
    part: ArmorPart,

    #[serde(rename = "sexType")]
    sex_type: SexType,

    names: HashMap<String, String>,
    rarity: i32,
    stat: ArmorStat,
    skills: Vec<ArmorSkill>,
}
