use std::{collections::HashMap, hash::Hash};

use itertools::{iproduct, izip};
use log::debug;

use crate::data::{deco::Decoration, skill::Skill};

use super::skill::MAX_SLOT_LEVEL;

#[derive(Default)]
pub struct DecorationCombinations {
    pub combinations: HashMap<String, Vec<Vec<Vec<i32>>>>,
}

#[derive(Clone, Debug)]
pub struct DecorationCombination {
    pub combs_per_skill: HashMap<String, Vec<i32>>,
    pub sum: Vec<i32>,
}

impl DecorationCombinations {
    pub fn new(
        decos_by_skill: &HashMap<String, Vec<Decoration>>,
        skills: &HashMap<String, Skill>,
    ) -> DecorationCombinations {
        let mut combinations = HashMap::<String, Vec<Vec<Vec<i32>>>>::new();

        for (id, decos) in decos_by_skill {
            let skill = skills.get(id).unwrap();
            let max_level = skill.max_level;

            if decos.len() == 1 {
                let mut skill_combs = Vec::new();

                let deco_skill_level = decos[0].skill_level;

                for req_level in 1..max_level + 1 {
                    let mut minimum_deco_count = req_level / deco_skill_level + 1;

                    if req_level % deco_skill_level == 0 {
                        minimum_deco_count -= 1;
                    }

                    skill_combs.push(vec![vec![minimum_deco_count]]);
                }

                combinations.insert(id.clone(), skill_combs);
            } else {
                combinations.insert(id.clone(), vec![]);

                let mut max_deco_counts = Vec::new();
                let mut init_case = Vec::new();

                for deco in decos {
                    let max_required = max_level / deco.skill_level;

                    max_deco_counts.push(max_required);
                    init_case.push(0);
                }

                for req_level in 1..max_level + 1 {
                    let mut skill_temp_combs = Vec::<Vec<i32>>::new();
                    let mut skill_done_combs = Vec::new();

                    skill_temp_combs.push(init_case.clone());

                    for (slot_size_index, max_deco_count) in max_deco_counts.iter().enumerate() {
                        let deco = &decos[slot_size_index];

                        let deco_temp_combs = skill_temp_combs.clone();

                        for temp_comb in &deco_temp_combs {
                            for count in (1..max_deco_count + 1).rev() {
                                let mut cur_level_sum: i32 = temp_comb.iter().sum();
                                cur_level_sum += count * deco.skill_level;

                                let mut next_temp_comb = temp_comb.clone();
                                next_temp_comb[slot_size_index] = count;

                                if req_level <= cur_level_sum {
                                    let mut has_better_slot_answer = false;

                                    for lower_deco_size in 0..slot_size_index {
                                        let lower_deco = &decos[lower_deco_size];

                                        let mut lower_level_sum: i32 = temp_comb.iter().sum();
                                        lower_level_sum += count * lower_deco.skill_level;

                                        if req_level <= lower_level_sum {
                                            has_better_slot_answer = true;
                                            break;
                                        }
                                    }

                                    if has_better_slot_answer == false {
                                        skill_done_combs.push(next_temp_comb);
                                    }
                                } else {
                                    skill_temp_combs.push(next_temp_comb);
                                }
                            }
                        }
                    }

                    combinations.get_mut(id).unwrap().push(skill_done_combs);
                }
            }
        }

        for (_, combs) in combinations.iter_mut() {
            for deco_size_combs in combs {
                let mut remove_comb_indices = Vec::new();

                'remove_loop: for index1 in 0..deco_size_combs.len() - 1 {
                    let deco_comb1 = &deco_size_combs[index1];

                    for index2 in index1 + 1..deco_size_combs.len() {
                        let deco_comb2 = &deco_size_combs[index2];

                        let mut is_inferior = true;

                        for (count1, count2) in izip!(deco_comb1, deco_comb2) {
                            if count1 < count2 {
                                is_inferior = false;
                                break;
                            }
                        }

                        if is_inferior {
                            remove_comb_indices.push(index1);
                            continue 'remove_loop;
                        }
                    }
                }

                for remove_index in remove_comb_indices.iter().rev() {
                    deco_size_combs.remove(*remove_index);
                }
            }
        }

        combinations = combinations
            .iter_mut()
            .map(|(skill_id, combs_per_skill)| {
                let mut ret = Vec::new();

                let decos = &decos_by_skill[skill_id];

                for combs_per_level in combs_per_skill {
                    let mut converted_level_combs = Vec::new();

                    for comb in combs_per_level {
                        let mut converted = Vec::new();
                        for _ in 0..MAX_SLOT_LEVEL {
                            converted.push(0);
                        }

                        for (deco_index, slot_count) in comb.iter().enumerate() {
                            let deco = &decos[deco_index];
                            let slot_size = deco.slot_size;

                            converted[(slot_size - 1) as usize] = *slot_count;
                        }

                        converted_level_combs.push(converted);
                    }

                    ret.push(converted_level_combs);
                }

                (skill_id.clone(), ret)
            })
            .collect();

        DecorationCombinations { combinations }
    }

    pub fn get(&self, skill_id: &String) -> Option<&Vec<Vec<Vec<i32>>>> {
        self.combinations.get(skill_id)
    }

    pub fn get_by_level(&self, skill_id: &String, req_level: i32) -> Vec<Vec<i32>> {
        match self.combinations.get(skill_id) {
            Some(val) => val[req_level as usize].clone(),
            None => Vec::new(),
        }
    }

    pub fn get_possible_combs(
        &self,
        req_skills: &HashMap<String, i32>,
    ) -> Vec<DecorationCombination> {
        if req_skills.len() == 0 {
            return Vec::new();
        }

        let mut all_possible_combs = Vec::<DecorationCombination>::new();

        let (skill_ids, combs_per_skill, mut level_indices) = self.get_iter_init_data(&req_skills);

        loop {
            let deco_comb = self.get_next_deco_comb(&req_skills, &skill_ids, &level_indices);

            all_possible_combs.push(deco_comb);

            let proceeded = self.proceed_next_iter(&mut level_indices, &combs_per_skill);

            if proceeded == false {
                break;
            }
        }

        all_possible_combs
    }

    pub fn has_possible_combs(
        &self,
        req_skills: &HashMap<String, i32>,
        armor_slots: &Vec<i32>,
    ) -> bool {
        self.iter_possible_combs(&req_skills, |deco_comb| {
            DecorationCombination::is_possible_static(armor_slots, &deco_comb.sum)
        })
    }

    pub fn iter_possible_combs<F>(&self, req_skills: &HashMap<String, i32>, mut f: F) -> bool
    where
        F: FnMut(DecorationCombination) -> bool,
    {
        if req_skills.len() == 0 {
            return true;
        }

        let (skill_ids, combs_per_skill, mut level_indices) = self.get_iter_init_data(&req_skills);

        let mut ret = false;

        loop {
            let deco_comb = self.get_next_deco_comb(&req_skills, &skill_ids, &level_indices);

            let result = f(deco_comb);

            if result == true {
                ret = true;
                break;
            }

            let proceeded = self.proceed_next_iter(&mut level_indices, &combs_per_skill);

            if proceeded == false {
                break;
            }
        }

        ret
    }

    fn get_iter_init_data<'a>(
        &self,
        req_skills: &'a HashMap<String, i32>,
    ) -> (Vec<&'a std::string::String>, Vec<Vec<Vec<i32>>>, Vec<usize>) {
        let skill_ids = req_skills
            .iter()
            .map(|(skill_id, _)| skill_id)
            .collect::<Vec<&String>>();

        let mut combs_per_skill = Vec::new();

        for &skill_id in &skill_ids {
            let level = req_skills[skill_id];
            let combs = self.combinations[skill_id][(level - 1) as usize].clone();

            combs_per_skill.push(combs);
        }

        let mut level_indices = Vec::<usize>::new();

        for _ in &combs_per_skill {
            level_indices.push(0);
        }

        (skill_ids, combs_per_skill, level_indices)
    }

    fn get_next_deco_comb(
        &self,
        req_skills: &HashMap<String, i32>,
        skill_ids: &Vec<&String>,
        level_indices: &Vec<usize>,
    ) -> DecorationCombination {
        let mut slot_combs = Vec::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slot_combs.push(0);
        }

        let mut all_skill_combs = HashMap::new();

        for (skill_index, &inside_level_index) in level_indices.iter().enumerate() {
            let skill_id = skill_ids[skill_index];
            let level_index = req_skills[skill_id] as usize - 1;
            let skill_comb = &self.combinations[skill_id][level_index][inside_level_index];

            all_skill_combs.insert(skill_id.clone(), skill_comb.clone());

            for (slot_size_index, count) in skill_comb.iter().enumerate() {
                slot_combs[slot_size_index] += count;
            }
        }

        DecorationCombination {
            combs_per_skill: all_skill_combs,
            sum: slot_combs,
        }
    }

    fn proceed_next_iter(
        &self,
        level_indices: &mut Vec<usize>,
        combs_per_skill: &Vec<Vec<Vec<i32>>>,
    ) -> bool {
        let mut promote = 0;

        for index in 0..level_indices.len() {
            let index = index as usize;

            level_indices[index as usize] += 1;

            if level_indices[index] == combs_per_skill[index].len() {
                level_indices[index] = 0;
                promote = 1;
            } else {
                promote = 0;
                break;
            }
        }

        promote == 0
    }

    pub fn compare(slots1: &Vec<i32>, slots2: &Vec<i32>) -> std::cmp::Ordering {
        for (slot1, slot2) in izip!(slots1, slots2) {
            if slot1 == slot2 {
                continue;
            }

            return slot1.cmp(slot2);
        }

        return std::cmp::Ordering::Equal;
    }
}

impl DecorationCombination {
    pub fn is_possible(&self, armor_slots: &Vec<i32>) -> bool {
        Self::is_possible_static(armor_slots, &self.sum)
    }

    pub fn is_possible_static(free_slots: &Vec<i32>, req_slots: &Vec<i32>) -> bool {
        let mut promote = 0;

        for (&free_slot, &req_slot) in izip!(free_slots, req_slots) {
            let req_slot = req_slot + promote;

            if req_slot == 0 {
                continue;
            }

            let taken = free_slot.min(req_slot);

            if taken == free_slot {
                promote += req_slot - taken;
            } else {
                promote = 0;
            }
        }

        promote == 0
    }

    // DO not execute on each part, only on full equipments
    pub fn is_possible_static_mut(free_slots: &mut Vec<i32>, req_slots: &mut Vec<i32>) -> bool {
        let mut promote = 0;

        for (free_slot, req_slot) in izip!(free_slots, req_slots) {
            *req_slot += promote;

            if *req_slot == 0 {
                continue;
            }

            let taken = (*free_slot).min(*req_slot);
            *free_slot -= taken;
            *req_slot -= taken;

            if *free_slot == 0 {
                promote = *req_slot;
            } else {
                promote = 0;
            }
        }

        promote == 0
    }
}
