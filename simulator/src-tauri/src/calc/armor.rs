use std::{cmp::Reverse, collections::HashMap};

use crate::data::{
    armor::{AnomalyArmor, ArmorPart, ArmorSkill, BaseArmor, SexType},
    deco::Decoration,
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
            slots: base.slots.clone(),
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
            slots: base.slots.clone(),
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

    pub fn calculate_point(
        &mut self,
        decos_possible: &HashMap<String, Vec<&Decoration>>,
        yes_deco_skills: &HashMap<String, i32>,
        no_deco_skills: &HashMap<String, i32>,
    ) {
        let mut point = 0;

        for (id, skill) in &self.skills {
            match yes_deco_skills.get(id) {
                Some(level) => {
                    let mut decos = decos_possible.get(id).unwrap().clone();
                    decos.sort_by_key(|deco| Reverse(deco.slot_size));
                    let max_slot_size = decos[0].slot_size;

                    point += skill.level.min(*level) * 2 * max_slot_size;
                }
                None => {}
            };
        }

        for (id, skill) in &self.skills {
            match no_deco_skills.get(id) {
                Some(level) => point += skill.level.min(*level) * 10,
                None => {}
            };
        }

        for (slot_size_index, count) in self.slots.iter().enumerate() {
            let slot_size = slot_size_index as i32 + 1;

            point += 3 * slot_size * count;
        }

        self.point = point;
    }
}
