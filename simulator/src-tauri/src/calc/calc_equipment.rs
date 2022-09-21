use std::{cmp::Reverse, collections::HashMap};

use crate::data::{armor::ArmorPart, deco::Decoration, skill::MAX_SLOT_LEVEL};

use super::{armor::CalcArmor, deco::CalcDeco, talisman::CalcTalisman};

pub trait CalcEquipment<'a> {
    fn id(&self) -> &String;
    fn skills(&self) -> &HashMap<String, i32>;
    fn mut_skills(&mut self) -> &mut HashMap<String, i32>;
    fn slots(&self) -> &Vec<i32>;
    fn part(&self) -> &ArmorPart;

    fn clone_dyn(&self) -> Box<dyn CalcEquipment<'a> + 'a>;

    fn as_armor(&self) -> &CalcArmor<'a>;
    fn as_talisman(&self) -> &CalcTalisman<'a>;

    fn is_le(&self, other: &dyn CalcEquipment<'a>) -> bool {
        let self_slots = self.slots();
        let other_slots = other.slots();

        for i in 0..MAX_SLOT_LEVEL {
            if self_slots[i] > other_slots[i] {
                return false;
            }
        }

        let self_skills = self.skills();
        let other_skills = other.skills();

        if self_skills.len() > other_skills.len() {
            return false;
        }

        let mut skill_condition_passed = 0;

        for (skill_id, level) in other_skills {
            let self_skill = self_skills.get(skill_id);

            if self_skill.is_none() {
                continue;
            }

            if level > self_skill.unwrap() {
                return false;
            } else {
                skill_condition_passed += 1;
            }
        }

        self_skills.len() == skill_condition_passed
    }

    fn get_point(
        &self,
        decos_possible: &HashMap<String, Vec<&Decoration>>,
        yes_deco_skills: &HashMap<String, i32>,
        no_deco_skills: &HashMap<String, i32>,
    ) -> i32 {
        let mut point = 0;

        for (id, &level) in self.skills() {
            match yes_deco_skills.get(id) {
                Some(&req_level) => {
                    let mut decos = decos_possible.get(id).unwrap().clone();
                    decos.sort_by_key(|deco| Reverse(deco.slot_size));

                    let max_slot_size = decos[0].slot_size;

                    point += level.min(req_level) * max_slot_size;
                }
                None => {
                    match no_deco_skills.get(id) {
                        Some(&req_level) => point += level.min(req_level) * 1000,
                        None => {}
                    };
                }
            };
        }

        point += CalcDeco::get_point(self.slots());

        point
    }

    fn subtract_skills(&mut self, req_skills: &mut HashMap<String, i32>) -> HashMap<String, i32> {
        let mut diffs = HashMap::new();

        for (id, level) in self.skills().clone() {
            let outer_skill = req_skills.get_mut(&id);

            if outer_skill.is_none() {
                continue;
            }

            let req_skill = outer_skill.unwrap();

            let taken = level.min(*req_skill);

            *req_skill -= taken;

            if *req_skill == 0 {
                req_skills.remove(&id);
            }

            diffs.insert(id, taken);
        }

        for (id, taken) in &diffs {
            let level = self.mut_skills().get_mut(id).unwrap();

            *level -= taken;

            if *level == 0 {
                self.mut_skills().remove(id);
            }
        }

        return diffs;
    }
}

impl<'a> Clone for Box<dyn CalcEquipment<'a> + 'a> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl<'a> PartialEq for dyn CalcEquipment<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.slots() == other.slots() && self.skills() == other.skills()
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}
