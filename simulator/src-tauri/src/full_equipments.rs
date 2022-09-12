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
struct SubSlotSkillCalculator {
    pub avail_slots: Vec<i32>,

    pub combinations: HashMap<String, Vec<i32>>,
}

impl<'a> SlotSkillCalculation<'a> {
    pub fn calculate(&mut self) {
        println!("Calculate begin...");

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
            return;
        }

        let mut temp_combinations = Vec::<SubSlotSkillCalculator>::new();
        let mut all_deco_combinations = Vec::<SubSlotSkillCalculator>::new();
        for (id, req_level) in &self.req_skills {
            println!("Calculate skill {} begin", id);

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

            println!(
                "Required level: {}, Max deco counts: {} {:?}",
                req_level, id, max_deco_counts
            );

            for (slot_size_index, max_deco_count) in max_deco_counts.iter().enumerate() {
                temp_combinations.push(self.get_sub());

                println!("Max deco: {}", max_deco_count);
                println!(
                    "{:?}, {:?}",
                    temp_combinations
                        .iter()
                        .map(|val| &val.combinations)
                        .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
                    temp_combinations
                        .iter()
                        .map(|val| &val.avail_slots)
                        .collect::<Vec<&Vec<i32>>>(),
                );

                let deco = decos[slot_size_index];

                let mut next_temp_combinations = Vec::<SubSlotSkillCalculator>::new();

                for temp_comb in &temp_combinations {
                    for count in (0..max_deco_count + 1).rev() {
                        println!(
                            "Count: {}, Max Deco: {}, Avail slots: {:?}",
                            count, max_deco_count, temp_comb.avail_slots
                        );

                        let mut cur_level_sum: i32 = temp_comb.combinations[id].iter().sum();
                        cur_level_sum += count * deco.skill_level;

                        println!(
                            "Current level: {}, {}, slot size: {}",
                            id, cur_level_sum, deco.slot_size
                        );

                        // TODO whether this skill is satified or not
                        if req_level <= cur_level_sum {
                            let mut next_temp_comb = temp_comb.clone();
                            next_temp_comb.combinations.get_mut(id).unwrap()[slot_size_index] =
                                count;

                            next_temp_combinations.push(next_temp_comb);
                        } else {
                        }
                    }
                }

                temp_combinations.clear();

                for temp_comb in next_temp_combinations.iter_mut() {
                    for local_slot_size_index in deco.slot_size - 1..MAX_SLOT_LEVEL - 1 {
                        let local_slot_size_index = local_slot_size_index as usize;

                        let left_slots = temp_comb.avail_slots[local_slot_size_index]
                            - temp_comb.combinations.get(id).unwrap()[slot_size_index];

                        if 0 <= left_slots {
                            temp_comb.avail_slots[local_slot_size_index] = left_slots;
                            temp_combinations.push(temp_comb.clone());
                        }
                    }
                }

                println!(
                    "{:?}, {:?}",
                    next_temp_combinations
                        .iter()
                        .map(|val| &val.combinations)
                        .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
                    next_temp_combinations
                        .iter()
                        .map(|val| &val.avail_slots)
                        .collect::<Vec<&Vec<i32>>>(),
                );

                println!(
                    "{:?}, {:?}",
                    temp_combinations
                        .iter()
                        .map(|val| &val.combinations)
                        .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
                    temp_combinations
                        .iter()
                        .map(|val| &val.avail_slots)
                        .collect::<Vec<&Vec<i32>>>(),
                );
            }

            println!();

            // println!("{:?}", all_deco_combinations);
        }

        for (_, level) in &self.req_skills {
            if 0 < *level {
                // TODO sone false condition
                break;
            }
        }

        // println!("{:?}, {:?}", self.all_skills, req_skills);
    }

    fn get_sub(&self) -> SubSlotSkillCalculator {
        let mut combinations = HashMap::<String, Vec<i32>>::new();

        for (id, _) in &self.req_skills {
            let mut skill_combs = Vec::new();

            let decos = self.decos_possible.get(id);

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
    ) -> bool {
        let mut avail_slots = self.avail_slots.clone();

        for (req_idx, req_slot) in req_slots.iter().enumerate() {
            let mut leftover_slot = *req_slot;

            for existing_idx in req_idx..avail_slots.len() {
                let existing_slot = avail_slots[existing_idx];

                if leftover_slot <= existing_slot {
                    avail_slots[existing_idx] -= leftover_slot;
                    break;
                } else {
                    avail_slots[existing_idx] = 0;
                    leftover_slot -= existing_slot;
                }
            }

            if 0 < leftover_slot {
                return false;
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

        calculator.calculate();

        return true;
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
