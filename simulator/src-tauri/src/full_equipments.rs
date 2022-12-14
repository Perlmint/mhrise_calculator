use std::collections::HashMap;

use crate::{
    data::{
        armor::ArmorPart,
        deco_combination::{DecorationCombination, DecorationCombinations},
        skill::MAX_SLOT_LEVEL,
    },
    BoxCalcEquipment,
};

#[derive(Clone)]
pub struct FullEquipments<'a> {
    pub weapon_slots: Vec<i32>,
    pub equipments: Vec<BoxCalcEquipment<'a>>,

    pub all_skills: HashMap<String, i32>,
    pub avail_slots: Vec<i32>,

    equipments_by_part: HashMap<ArmorPart, BoxCalcEquipment<'a>>,
}

impl<'a> FullEquipments<'a> {
    pub fn new(weapon_slots: Vec<i32>, equipments: Vec<BoxCalcEquipment<'a>>) -> FullEquipments {
        let equipments_by_part = Self::save_by_part_clone(&equipments);
        let (all_skills, avail_slots) = Self::calculate_skills_slots(&weapon_slots, &equipments);

        FullEquipments {
            weapon_slots,
            equipments,
            all_skills,
            avail_slots,
            equipments_by_part,
        }
    }

    pub fn get_by_part(&self, part: &ArmorPart) -> &BoxCalcEquipment<'a> {
        &self.equipments_by_part[part]
    }

    pub fn get_possible_combs(
        &self,
        mut req_skills: HashMap<String, i32>,
        req_slots: &Vec<i32>,
        no_deco_skills: &HashMap<String, i32>,
        deco_comb_calculator: &DecorationCombinations,
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

    pub fn contains_skills(&self, req_skills: &HashMap<String, i32>) -> bool {
        for (id, req_level) in req_skills.clone() {
            let existing = self.all_skills.get(&id);

            if existing.is_some() {
                if req_level - existing.unwrap() <= 0 {
                    continue;
                }
            }

            return false;
        }

        true
    }

    pub fn calculate_skills_slots(
        weapon_slots: &Vec<i32>,
        equipments: &Vec<BoxCalcEquipment<'a>>,
    ) -> (HashMap<String, i32>, Vec<i32>) {
        let mut skills = HashMap::<String, i32>::new();
        let mut slots = Vec::<i32>::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for equip in equipments {
            for (id, &level) in equip.skills() {
                let existing = skills.get(id);

                let mut level_sum = level;

                if existing.is_some() {
                    level_sum += existing.unwrap();
                }

                skills.insert(id.clone(), level_sum);
            }

            for (slot_size_index, count) in equip.slots().iter().enumerate() {
                if *count == 0 {
                    continue;
                }

                slots[slot_size_index] += *count;
            }
        }

        for weapon_slot in weapon_slots {
            if *weapon_slot == 0 {
                continue;
            }

            slots[(weapon_slot - 1) as usize] += 1;
        }

        return (skills, slots);
    }

    // TODO: why lifetime error?
    pub fn _save_by_part(
        equipments: &'a Vec<BoxCalcEquipment<'a>>,
    ) -> HashMap<ArmorPart, &'a BoxCalcEquipment<'a>> {
        let mut equipments_by_part = HashMap::new();

        for equipment in equipments {
            equipments_by_part.insert(equipment.part().clone(), equipment);
        }

        equipments_by_part
    }

    pub fn save_by_part_clone(
        equipments: &Vec<BoxCalcEquipment<'a>>,
    ) -> HashMap<ArmorPart, BoxCalcEquipment<'a>> {
        let mut equipments_by_part = HashMap::new();

        for equipment in equipments {
            equipments_by_part.insert(equipment.part().clone(), equipment.clone());
        }

        equipments_by_part
    }

    pub fn get_full_equip_id(equipments: &Vec<&BoxCalcEquipment<'a>>) -> String {
        let mut equipments_by_part = HashMap::new();

        for equipment in equipments {
            equipments_by_part.insert(equipment.part().clone(), equipment);
        }

        Self::format_full_equip_id(
            equipments_by_part[&ArmorPart::Helm].id(),
            equipments_by_part[&ArmorPart::Torso].id(),
            equipments_by_part[&ArmorPart::Arm].id(),
            equipments_by_part[&ArmorPart::Waist].id(),
            equipments_by_part[&ArmorPart::Feet].id(),
            equipments_by_part[&ArmorPart::Talisman].id(),
        )
    }

    fn format_full_equip_id(
        helm: &String,
        torso: &String,
        arm: &String,
        waist: &String,
        feet: &String,
        tali: &String,
    ) -> String {
        format!(
            "FULLEQUIP-{}-{}-{}-{}-{}-{}",
            helm, torso, arm, waist, feet, tali,
        )
    }
}
