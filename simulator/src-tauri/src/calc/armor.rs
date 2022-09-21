use std::{cmp::Reverse, collections::HashMap};

use log::debug;

use crate::data::{
    armor::{AnomalyArmor, ArmorPart, ArmorSkill, BaseArmor, SexType},
    deco::Decoration,
    deco_combination::DecorationCombination,
    skill::MAX_SLOT_LEVEL,
};

use super::{calc_equipment::CalcEquipment, deco::CalcDeco, talisman::CalcTalisman};

#[derive(Clone, Debug)]
pub struct CalcArmor<'a> {
    base: &'a BaseArmor,
    anomaly: Option<&'a AnomalyArmor>,

    part: ArmorPart,
    sex_type: SexType,

    rarity: i32,
    skills: HashMap<String, i32>,
    slots: Vec<i32>,
}

impl<'a> CalcArmor<'a> {
    pub fn new(base: &'a BaseArmor) -> Self {
        let mut skills = HashMap::new();

        for (skill_id, armor_skill) in &base.skills {
            skills.insert(skill_id.clone(), armor_skill.level);
        }

        Self {
            base,
            anomaly: None,
            part: base.part.clone(),
            sex_type: base.sex_type.clone(),
            rarity: base.rarity,
            skills,
            slots: Self::convert_from_base_slots(&base.slots),
        }
    }

    pub fn names(&self) -> &HashMap<String, String> {
        &self.base.names
    }

    pub fn sex_type(&self) -> &SexType {
        &self.base.sex_type
    }

    pub fn rarity(&self) -> i32 {
        self.base.rarity
    }

    pub fn subtract_slots(&mut self, single_deco_skills: &mut Vec<(&String, i32, i32)>) -> bool {
        let mut success = true;

        for (_, slot_size, count) in single_deco_skills.iter_mut() {
            if *count == 0 {
                continue;
            }

            let init_size_index = *slot_size - 1;

            let mut promote = 0;

            for slot_size_index in init_size_index..MAX_SLOT_LEVEL {
                let slot_size_index = slot_size_index as usize;

                let taken = (*count).min(self.slots[slot_size_index]);

                self.slots[slot_size_index] -= taken;
                *count -= taken;

                promote = *count;

                if promote == 0 {
                    break;
                }
            }

            if promote != 0 {
                success = false;
                break;
            }
        }

        success
    }

    fn convert_from_base_slots(base_slots: &Vec<i32>) -> Vec<i32> {
        let mut ret = Vec::new();

        for _ in 0..MAX_SLOT_LEVEL {
            ret.push(0);
        }

        for slot_size in base_slots {
            if *slot_size == 0 {
                continue;
            }

            ret[*slot_size as usize - 1] += 1;
        }

        ret
    }
}

impl<'a> CalcEquipment<'a> for CalcArmor<'a> {
    fn id(&self) -> &String {
        &self.base.id()
    }

    fn skills(&self) -> &HashMap<String, i32> {
        &self.skills
    }

    fn mut_skills(&mut self) -> &mut HashMap<String, i32> {
        &mut self.skills
    }

    fn slots(&self) -> &Vec<i32> {
        &self.slots
    }

    fn part(&self) -> &ArmorPart {
        &self.part
    }

    fn as_armor(&self) -> &CalcArmor<'a> {
        self
    }

    fn as_talisman(&self) -> &CalcTalisman<'a> {
        panic!("This is not talisman")
    }
}
