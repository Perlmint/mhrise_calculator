use std::{cmp::Reverse, collections::HashMap};

use crate::data::{
    armor::{AnomalyArmor, ArmorPart, ArmorSkill, BaseArmor, SexType},
    deco::Decoration,
    deco_combination::DecorationCombination,
    skill::MAX_SLOT_LEVEL,
};

#[derive(Clone, Debug)]
pub struct CalcArmor<'a> {
    base: &'a BaseArmor,
    anomaly: Option<&'a AnomalyArmor>,

    part: ArmorPart,
    sex_type: SexType,

    rarity: i32,
    skills: HashMap<String, ArmorSkill>,
    slots: Vec<i32>,

    point: i32,
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
            point: 0,
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
            point: 0,
        }
    }

    pub fn id(&self) -> &String {
        &self.base.id
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

    pub fn point(&self) -> i32 {
        self.point
    }

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

        DecorationCombination::is_possible_static_mut(&mut self.slots, req_slots);

        return diffs;
    }

    pub fn subtract_slots(&mut self, single_deco_skills: &mut HashMap<String, (i32, i32)>) {
        for (_, (slot_size, count)) in single_deco_skills {
            let slot_size_index = *slot_size as usize - 1;

            let taken = (*count).min(self.slots[slot_size_index]);

            self.slots[slot_size_index] -= taken;
            *count -= taken;
        }
    }

    pub fn calculate_point(
        &mut self,
        decos_possible: &HashMap<String, Vec<&Decoration>>,
        yes_deco_skills: &HashMap<String, i32>,
        no_deco_skills: &HashMap<String, i32>,
    ) {
        self.point = self.get_point(decos_possible, yes_deco_skills, no_deco_skills);
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

        for (slot_size_index, count) in self.slots.iter().enumerate() {
            let slot_level = slot_size_index as i32 + 1;

            let slot_point;

            if slot_level == 4 {
                slot_point = slot_level + 2;
            } else {
                slot_point = slot_level
            }

            point += slot_point * count;
        }

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
