use std::collections::HashMap;

use crate::{
    calc::armor::CalcArmor,
    calc::calc_equipment::CalcEquipment,
    data::{
        armor::{ArmorPart, Talisman},
        deco_combination::{DecorationCombination, DecorationCombinations},
        skill::MAX_SLOT_LEVEL,
    },
};

#[derive(Debug, Default, Clone)]
pub struct FullEquipments<'a> {
    pub weapon_slots: Vec<i32>,
    pub armors: Vec<CalcArmor<'a>>,
    pub talisman: Option<&'a Talisman>,

    pub all_skills: HashMap<String, i32>,
    pub avail_slots: Vec<i32>,

    armor_by_part: HashMap<ArmorPart, CalcArmor<'a>>,
    id: String,
}

impl<'a> FullEquipments<'a> {
    pub fn new(
        weapon_slots: Vec<i32>,
        armors: Vec<CalcArmor<'a>>,
        talisman: Option<&'a Talisman>,
    ) -> FullEquipments<'a> {
        let mut ret = FullEquipments {
            weapon_slots,
            armors,
            talisman,
            ..Default::default()
        };

        (ret.all_skills, ret.avail_slots) = ret.sum();

        let mut armors_by_part = HashMap::<ArmorPart, CalcArmor<'a>>::new();

        for armor in &ret.armors {
            armors_by_part.insert(armor.part(), armor.clone());
        }

        ret.armor_by_part = armors_by_part;

        ret.id = format!(
            "FULLEQUIP-{}-{}-{}-{}-{}",
            ret.get_by_part(&ArmorPart::Helm).id(),
            ret.get_by_part(&ArmorPart::Torso).id(),
            ret.get_by_part(&ArmorPart::Arm).id(),
            ret.get_by_part(&ArmorPart::Waist).id(),
            ret.get_by_part(&ArmorPart::Feet).id(),
        );

        ret
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn get_by_part(&self, part: &ArmorPart) -> &CalcArmor<'a> {
        &self.armor_by_part[part]
    }

    pub fn get_possible_combs(
        &self,
        mut req_skills: HashMap<String, i32>,
        req_slots: &Vec<i32>,
        no_deco_skills: &HashMap<String, i32>,
        deco_comb_calculator: &'a DecorationCombinations,
    ) -> (bool, Vec<DecorationCombination>) {
        let mut avail_slots = self.avail_slots.clone();

        let slot_available =
            DecorationCombination::is_possible_static_mut(&mut avail_slots, &mut req_slots.clone());

        if slot_available == false {
            return (false, Vec::new());
        }

        let mut remove_ids = Vec::new();

        for (id, level) in req_skills.clone() {
            let existing = self.all_skills.get(&id);

            if existing.is_some() {
                if level - existing.unwrap() <= 0 {
                    remove_ids.push(id);
                }
            }
        }

        for id in remove_ids {
            req_skills.remove(&id);
        }

        if req_skills.len() == 0 {
            return (
                true,
                vec![DecorationCombination {
                    combs_per_skill: HashMap::new(),
                    sum: Vec::new(),
                }],
            );
        }

        for (id, _) in &req_skills {
            if no_deco_skills.contains_key(id) {
                return (false, Vec::new());
            }
        }

        let mut req_deco_combs = deco_comb_calculator.get_possible_combs(&req_skills);
        req_deco_combs.retain(|comb| comb.is_possible(&avail_slots));

        (req_deco_combs.len() != 0, req_deco_combs)
    }

    pub fn subtract_slots(&mut self, req_slots: &mut Vec<i32>) -> bool {
        DecorationCombination::is_possible_static_mut(&mut self.avail_slots, req_slots)
    }

    fn sum(&self) -> (HashMap<String, i32>, Vec<i32>) {
        let mut skills = HashMap::<String, i32>::new();
        let mut slots = Vec::<i32>::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for armor in &self.armors {
            for (id, &level) in armor.skills() {
                let existing = skills.get(id);

                let mut level_sum = level;

                if existing.is_some() {
                    level_sum += existing.unwrap();
                }

                skills.insert(id.clone(), level_sum);
            }

            for (slot_size_index, count) in armor.slots().iter().enumerate() {
                if *count == 0 {
                    continue;
                }

                slots[slot_size_index] += *count;
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
