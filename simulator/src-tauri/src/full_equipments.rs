use std::collections::HashMap;

use crate::data::{
    armor::{ArmorPart, BaseArmor, Talisman},
    deco::Decoration,
    skill::MAX_SLOT_LEVEL,
};

#[derive(Default, Clone)]
pub struct FullEquipments<'a> {
    pub weapon_slots: Vec<i32>,
    pub armors: HashMap<ArmorPart, &'a BaseArmor>,
    pub talisman: Option<&'a Talisman>,

    pub all_skills: HashMap<String, i32>,
    pub avail_slots: Vec<i32>,
}

#[derive(Clone, Debug)]
struct SlotSkillCalculation<'a> {
    pub avail_slots: Vec<i32>,
    pub req_skills: HashMap<String, i32>,
    pub decos_possible: &'a HashMap<String, Vec<&'a Decoration>>,
}

#[derive(Clone, Debug)]
pub struct SubSlotSkillCalculator {
    pub avail_slots: Vec<i32>,

    pub combinations: HashMap<String, Vec<i32>>,
}

impl<'a> SlotSkillCalculation<'a> {
    pub fn calculate(&mut self) -> Vec<SubSlotSkillCalculator> {
        // println!("Calculate begin... {:?}", self.avail_slots);

        let mut is_possible = true;

        for (id, req_level) in &self.req_skills {
            let req_level = *req_level;

            if req_level <= 0 {
                continue;
            }

            let decos = &self.decos_possible.get(id);

            if decos.is_none() {
                is_possible = false;
                break;
            }

            let decos = decos.unwrap();

            if decos.len() == 1 {
                let size = decos[0].slot_size;

                self.avail_slots[(size - 1) as usize] -= req_level;

                if self.avail_slots[(size - 1) as usize] < 0 {
                    is_possible = false;
                    break;
                }
            }
        }

        if is_possible == false {
            return Vec::new();
        }

        let mut all_temp_combinations = Vec::<SubSlotSkillCalculator>::new();

        all_temp_combinations.push(self.get_sub());

        let mut idx = 0;

        for (id, req_level) in &self.req_skills {
            let req_level = *req_level;

            if req_level <= 0 {
                continue;
            }

            let decos = self.decos_possible.get(id).unwrap();

            if decos.len() == 1 {
                continue;
            }

            let mut max_deco_counts = Vec::new();

            for deco in decos {
                let max_required = req_level / deco.skill_level;

                max_deco_counts.push(max_required);
            }

            let mut skill_temp_combs = all_temp_combinations.clone();
            let mut skill_done_combs = Vec::new();

            for (slot_size_index, max_deco_count) in max_deco_counts.iter().enumerate() {
                let deco = decos[slot_size_index];

                let deco_temp_combs = skill_temp_combs.clone();

                for temp_comb in &deco_temp_combs {
                    for count in (1..max_deco_count + 1).rev() {
                        let mut cur_level_sum: i32 = temp_comb.combinations[id].iter().sum();
                        cur_level_sum += count * deco.skill_level;

                        let mut next_temp_comb = temp_comb.clone();
                        next_temp_comb.combinations.get_mut(id).unwrap()[slot_size_index] = count;

                        if req_level <= cur_level_sum {
                            let mut has_better_slot_answer = false;

                            for lower_deco_size in 0..slot_size_index {
                                let lower_deco = decos[lower_deco_size];

                                let mut lower_level_sum: i32 =
                                    temp_comb.combinations[id].iter().sum();
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

            // println!(
            //     "{} {} skill end check: req_level - {}, {:?}, {:?}",
            //     idx,
            //     id,
            //     req_level,
            //     skill_done_combs
            //         .iter()
            //         .map(|val| &val.combinations)
            //         .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
            //     skill_done_combs
            //         .iter()
            //         .map(|val| &val.avail_slots)
            //         .collect::<Vec<&Vec<i32>>>(),
            // );

            all_temp_combinations = skill_done_combs
                .iter_mut()
                .filter_map(|comb| {
                    let skill_combs = comb.combinations.get(id).unwrap();

                    for (slot_size_index, deco) in decos.iter().enumerate() {
                        let mut deco_count_required = skill_combs[slot_size_index];

                        if deco_count_required == 0 {
                            continue;
                        }

                        for avail_slot_size_index in deco.slot_size - 1..MAX_SLOT_LEVEL {
                            let avail_slot_size_index = avail_slot_size_index as usize;

                            let taken =
                                deco_count_required.min(comb.avail_slots[avail_slot_size_index]);

                            comb.avail_slots[avail_slot_size_index] -= taken;
                            deco_count_required -= taken;

                            if deco_count_required == 0 {
                                break;
                            }
                        }

                        if 0 < deco_count_required {
                            return None;
                        }
                    }

                    return Some(comb.clone());
                })
                .collect();

            // println!(
            //     "{} {} skill end check: {:?}, {:?}",
            //     idx,
            //     id,
            //     all_temp_combinations
            //         .iter()
            //         .map(|val| &val.combinations)
            //         .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
            //     all_temp_combinations
            //         .iter()
            //         .map(|val| &val.avail_slots)
            //         .collect::<Vec<&Vec<i32>>>(),
            // );

            idx += 1;
        }

        return all_temp_combinations;
    }

    fn get_sub(&self) -> SubSlotSkillCalculator {
        let mut combinations = HashMap::<String, Vec<i32>>::new();

        for (id, count) in &self.req_skills {
            if *count == 0 {
                continue;
            }

            let mut skill_combs = Vec::new();

            let decos = self.decos_possible.get(id);

            if decos.is_none() {
                continue;
            }

            for _ in decos.unwrap() {
                skill_combs.push(0);
            }

            combinations.insert(id.clone(), skill_combs);
        }

        SubSlotSkillCalculator {
            avail_slots: self.avail_slots.clone(),

            combinations,
        }
    }
}

impl<'a> FullEquipments<'a> {
    pub fn new(
        weapon_slots: Vec<i32>,
        armors: HashMap<ArmorPart, &'a BaseArmor>,
        talisman: Option<&'a Talisman>,
    ) -> FullEquipments<'a> {
        let mut ret = FullEquipments {
            weapon_slots,
            armors,
            talisman,
            ..Default::default()
        };

        (ret.all_skills, ret.avail_slots) = ret.sum();

        ret
    }
    pub fn is_possible(
        &self,
        mut req_skills: HashMap<String, i32>,
        req_slots: &Vec<i32>,
        decos_possible: &HashMap<String, Vec<&Decoration>>,
    ) -> Vec<SubSlotSkillCalculator> {
        let mut avail_slots = self.avail_slots.clone();

        for (req_idx, req_slot_count) in req_slots.iter().enumerate() {
            let mut req_count = *req_slot_count;

            if req_count == 0 {
                continue;
            }

            for existing_idx in req_idx..avail_slots.len() {
                let avail_count = avail_slots[existing_idx];

                let taken_count = req_count.min(avail_count);

                req_count -= taken_count;
                avail_slots[existing_idx] -= taken_count;

                if req_count == 0 {
                    break;
                }
            }

            if 0 < req_count {
                return Vec::new();
            }
        }

        for (id, level) in req_skills.iter_mut() {
            let existing = self.all_skills.get(id);

            if existing.is_some() {
                *level -= existing.unwrap();
            }
        }

        let mut calculator = SlotSkillCalculation {
            avail_slots: avail_slots.clone(),
            req_skills: req_skills.clone(),
            decos_possible,
        };

        let all_combs = calculator.calculate();

        return all_combs;
    }

    fn sum(&self) -> (HashMap<String, i32>, Vec<i32>) {
        let mut skills = HashMap::<String, i32>::new();
        let mut slots = Vec::<i32>::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for armor in &self.armors {
            for (id, skill_info) in &armor.1.skills {
                let existing = skills.get(id);

                let mut level_sum = skill_info.level;

                if existing.is_some() {
                    level_sum += existing.unwrap();
                }

                skills.insert(id.clone(), level_sum);
            }

            for (_, slot_size) in armor.1.slots.iter().enumerate() {
                if *slot_size == 0 {
                    continue;
                }

                slots[(slot_size - 1) as usize] += 1;
            }
        }

        for weapon_slot in &self.weapon_slots {
            if *weapon_slot == 0 {
                continue;
            }

            slots[(weapon_slot - 1) as usize] += 1;
        }

        if self.talisman.is_some() {
            for skill in &self.talisman.unwrap().skills {
                let id = &skill.id;
                let level = skill.level;

                let existing = skills.get_mut(id);

                if existing.is_some() {
                    *existing.unwrap() += level;
                } else {
                    skills.insert(id.clone(), level);
                }
            }

            for tali_slot in &self.talisman.unwrap().slot_sizes {
                if *tali_slot == 0 {
                    continue;
                }

                slots[(tali_slot - 1) as usize] += 1;
            }
        }

        return (skills, slots);
    }
}
