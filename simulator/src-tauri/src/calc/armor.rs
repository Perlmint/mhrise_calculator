use std::{cmp::Reverse, collections::HashMap};

use log::debug;

use crate::data::{
    armor::{AnomalyArmor, ArmorPart, ArmorSkill, BaseArmor, SexType},
    deco::Decoration,
    deco_combination::DecorationCombination,
    skill::MAX_SLOT_LEVEL,
};

use super::deco::CalcDeco;

#[derive(Clone, Debug)]
pub struct CalcArmor<'a> {
    base: &'a BaseArmor,
    anomaly: Option<&'a AnomalyArmor>,

    part: ArmorPart,
    sex_type: SexType,

    rarity: i32,
    skills: HashMap<String, ArmorSkill>,
    slots: Vec<i32>,
}

impl<'a> CalcArmor<'a> {
    pub fn new(base: &'a BaseArmor) -> Self {
        Self {
            base,
            anomaly: None,
            part: base.part.clone(),
            sex_type: base.sex_type.clone(),
            rarity: base.rarity,
            skills: base.skills.clone(),
            slots: Self::convert_from_base_slots(&base.slots),
        }
    }

    pub fn new_anomaly(anomaly: &'a AnomalyArmor) -> Self {
        let base = &anomaly.affected;

        Self {
            base,
            anomaly: Option::Some(anomaly),
            part: base.part.clone(),
            sex_type: base.sex_type.clone(),
            rarity: base.rarity,
            skills: base.skills.clone(),
            slots: Self::convert_from_base_slots(&base.slots),
        }
    }

    pub fn id(&self) -> &String {
        &self.base.id
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

    pub fn part(&self) -> ArmorPart {
        self.part.clone()
    }

    pub fn skills(&self) -> &HashMap<String, ArmorSkill> {
        &self.skills
    }

    pub fn slots(&self) -> &Vec<i32> {
        &self.slots
    }

    pub fn subtract_skills(
        &mut self,
        req_skills: &mut HashMap<String, i32>,
    ) -> HashMap<String, i32> {
        let mut diffs = HashMap::new();

        for (id, skill) in self.skills.clone() {
            let outer_skill = req_skills.get_mut(&id);

            if outer_skill.is_none() {
                continue;
            }

            let req_skill = outer_skill.unwrap();

            let taken = skill.level.min(*req_skill);

            *req_skill -= taken;

            if *req_skill == 0 {
                req_skills.remove(&id);
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

        return diffs;
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

    pub fn get_point(
        &self,
        decos_possible: &HashMap<String, Vec<&Decoration>>,
        yes_deco_skills: &HashMap<String, i32>,
        no_deco_skills: &HashMap<String, i32>,
    ) -> i32 {
        let mut point = 0;

        for (id, skill) in &self.skills {
            match yes_deco_skills.get(id) {
                Some(level) => {
                    let mut decos = decos_possible.get(id).unwrap().clone();
                    decos.sort_by_key(|deco| Reverse(deco.slot_size));

                    let max_slot_size = decos[0].slot_size;

                    point += skill.level.min(*level) * max_slot_size;
                }
                None => {
                    match no_deco_skills.get(id) {
                        Some(level) => point += skill.level.min(*level) * 1000,
                        None => {}
                    };
                }
            };
        }

        point += CalcDeco::get_point(&self.slots);

        point
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
