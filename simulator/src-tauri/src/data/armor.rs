use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tauri::regex::Regex;

pub static EMPTY_ARMOR_PREFIX: &str = "__empty";
pub static SLOT_ARMOR_PREFIX: &str = "__slot";

lazy_static! {
    pub static ref SLOT_ARMOR_REGEX: Regex =
        Regex::new(&format!("{}_{}", SLOT_ARMOR_PREFIX, r"(\d+)-(\d+)-(\d+)")).unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
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

impl ArmorPart {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArmorPart::Helm => "helm",
            ArmorPart::Torso => "torso",
            ArmorPart::Arm => "arm",
            ArmorPart::Waist => "waist",
            ArmorPart::Feet => "feet",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    pub skills: HashMap<String, ArmorSkill>,
    pub slots: Vec<i32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct AnomalyArmor {
    pub original: BaseArmor,
    pub affected: BaseArmor,

    #[serde(rename = "statDiff")]
    pub stat_diff: ArmorStat,

    #[serde(rename = "slotDiffs")]
    pub slot_diffs: Vec<i32>,

    #[serde(rename = "skillDiffs")]
    pub skill_diffs: HashMap<String, ArmorSkill>,
}

pub struct TalismanSkill {
    pub id: String,
    pub level: i32,
}

#[derive(Default)]
pub struct Talisman {
    pub skills: Vec<TalismanSkill>,
    pub slot_sizes: Vec<i32>,
}

impl ArmorPart {
    pub fn get_all() -> Vec<Self> {
        return vec![Self::Helm, Self::Torso, Self::Arm, Self::Waist, Self::Feet];
    }
}

impl BaseArmor {
    pub fn subtract_skills(
        &mut self,
        outer_skills: &mut HashMap<String, i32>,
        req_slots: &mut Vec<i32>,
    ) -> HashMap<String, i32> {
        let mut diffs = HashMap::new();

        for (id, skill) in self.skills.clone() {
            let outer_skill = outer_skills.get_mut(&id);

            if outer_skill.is_none() {
                continue;
            }

            let outer_skill = outer_skill.unwrap();

            let taken = skill.level.min(*outer_skill);

            *outer_skill -= taken;

            if *outer_skill == 0 {
                outer_skills.remove(&id);
            }

            diffs.insert(id, taken);
        }

        for (id, taken) in &diffs {
            let skill = self.skills.get_mut(id).unwrap();

            skill.level -= taken;

            if skill.level == 0 {
                self.skills.remove(id);
            }
        }

        for slot_size in &self.slots {
            let slot_size = *slot_size as usize;

            if slot_size == 0 {
                continue;
            }

            let req_count_leftover = req_slots[slot_size - 1];

            if 0 < req_count_leftover {
                req_slots[slot_size - 1] -= 1;
            }
        }

        return diffs;
    }

    pub fn create_empty(part: ArmorPart) -> BaseArmor {
        Self {
            id: format!("{}_{}", EMPTY_ARMOR_PREFIX, part.as_str()),
            names: HashMap::new(),
            part,
            rarity: 10,
            sex_type: SexType::All,
            skills: HashMap::new(),
            slots: vec![0, 0, 0],
            stat: ArmorStat {
                defense: 0,
                fire_res: 0,
                water_res: 0,
                ice_res: 0,
                elec_res: 0,
                dragon_res: 0,
            },
        }
    }

    pub fn get_slot_armor_id(armor: &BaseArmor) -> String {
        format!(
            "{}_{}-{}-{}",
            SLOT_ARMOR_PREFIX, armor.slots[0], armor.slots[1], armor.slots[2]
        )
    }

    pub fn parse_slot_armor_id(slot_armor_id: &String) -> Vec<i32> {
        let mut ret = Vec::new();

        for cap in SLOT_ARMOR_REGEX.captures_iter(slot_armor_id) {
            ret.push(cap.get(1).unwrap().as_str().parse::<i32>().unwrap());
            ret.push(cap.get(2).unwrap().as_str().parse::<i32>().unwrap());
            ret.push(cap.get(3).unwrap().as_str().parse::<i32>().unwrap());
        }

        ret
    }

    pub fn get_slot_armor(part: ArmorPart, slot_armor_id: String) -> BaseArmor {
        Self {
            id: slot_armor_id.clone(),
            names: HashMap::new(),
            part,
            rarity: 10,
            sex_type: SexType::All,
            skills: HashMap::new(),
            slots: BaseArmor::parse_slot_armor_id(&slot_armor_id),
            stat: ArmorStat {
                defense: 0,
                fire_res: 0,
                water_res: 0,
                ice_res: 0,
                elec_res: 0,
                dragon_res: 0,
            },
        }
    }
}

impl AnomalyArmor {
    pub fn new(
        original: BaseArmor,
        stat_diff: ArmorStat,
        slot_diffs: Vec<i32>,
        skill_diffs: HashMap<String, ArmorSkill>,
    ) -> AnomalyArmor {
        let mut affected = original.clone();

        affected.stat.defense += stat_diff.defense;
        affected.stat.fire_res += stat_diff.fire_res;
        affected.stat.water_res += stat_diff.water_res;
        affected.stat.ice_res += stat_diff.ice_res;
        affected.stat.elec_res += stat_diff.elec_res;
        affected.stat.dragon_res += stat_diff.dragon_res;

        for (id, skill_info) in &skill_diffs {
            let diff_level = skill_info.level;

            let new_value;
            let existing_skill = affected.skills.get(id);

            if existing_skill.is_some() {
                let old_value = existing_skill.unwrap();
                new_value = ArmorSkill {
                    level: old_value.level + diff_level,
                };
            } else {
                new_value = ArmorSkill { level: diff_level }
            }

            affected.skills.insert(id.clone(), new_value);
        }

        for (index, slot_diff) in slot_diffs.iter().enumerate() {
            if affected.slots.len() <= index {
                affected.slots[index] += slot_diff;
            } else {
                affected.slots.push(*slot_diff);
            }
        }

        AnomalyArmor {
            original,
            affected,
            stat_diff,
            slot_diffs,
            skill_diffs,
        }
    }
}
