use std::collections::HashMap;

use crate::data::{armor::Talisman, skill::MAX_SLOT_LEVEL};

use super::calc_equipment::CalcEquipment;

pub struct CalcTalisman<'a> {
    tali: &'a Talisman,

    slots: Vec<i32>,
    skills: HashMap<String, i32>,
}

impl<'a> CalcTalisman<'a> {
    pub fn new(tali: &'a Talisman) -> Self {
        let mut slots = Vec::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for &slot_size in &tali.slot_sizes {
            if slot_size == 0 {
                continue;
            }

            let slot_size_index = slot_size as usize - 1;
            slots[slot_size_index] += 1;
        }

        let mut skills = HashMap::new();

        for tali_skill in &tali.skills {
            skills.insert(tali_skill.id.clone(), tali_skill.level);
        }

        Self {
            tali,
            slots,
            skills,
        }
    }
}

impl<'a> CalcEquipment for CalcTalisman<'a> {
    fn slots(&self) -> &Vec<i32> {
        &self.slots
    }

    fn skills(&self) -> &HashMap<String, i32> {
        &self.skills
    }
}
