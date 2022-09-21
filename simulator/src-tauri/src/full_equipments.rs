use std::{collections::HashMap, marker::PhantomData};

use crate::{
    calc::calc_equipment::CalcEquipment,
    data::{
        armor::ArmorPart,
        deco_combination::{DecorationCombination, DecorationCombinations},
        skill::MAX_SLOT_LEVEL,
    },
};

#[derive(Clone)]
pub struct FullEquipments<'a> {
    pub weapon_slots: Vec<i32>,
    pub equipments: Vec<Box<dyn CalcEquipment<'a>>>,

    pub all_skills: HashMap<String, i32>,
    pub avail_slots: Vec<i32>,

    equipments_by_part: HashMap<ArmorPart, Box<dyn CalcEquipment<'a>>>,
    id: String,
    phantom: PhantomData<&'a i32>,
}

impl<'a> FullEquipments<'a> {
    pub fn new(
        weapon_slots: Vec<i32>,
        equipments: Vec<Box<dyn CalcEquipment<'a>>>,
    ) -> FullEquipments<'a> {
        let mut ret = FullEquipments {
            weapon_slots,
            equipments,
            all_skills: Default::default(),
            avail_slots: Default::default(),
            equipments_by_part: Default::default(),
            id: Default::default(),
            phantom: Default::default(),
        };

        (ret.all_skills, ret.avail_slots) = ret.sum();

        let mut equipments_by_part = HashMap::<ArmorPart, Box<dyn CalcEquipment<'a>>>::new();

        for equipment in ret.equipments() {
            equipments_by_part.insert(equipment.part().clone(), equipment);
        }

        ret.equipments_by_part = equipments_by_part;

        ret.id = format!(
            "FULLEQUIP-{}-{}-{}-{}-{}-{}",
            ret.get_by_part(&ArmorPart::Helm).id(),
            ret.get_by_part(&ArmorPart::Torso).id(),
            ret.get_by_part(&ArmorPart::Arm).id(),
            ret.get_by_part(&ArmorPart::Waist).id(),
            ret.get_by_part(&ArmorPart::Feet).id(),
            ret.get_by_part(&ArmorPart::Talisman).id(),
        );

        ret
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn get_by_part(&self, part: &ArmorPart) -> &'a Box<dyn CalcEquipment<'a>> {
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

    pub fn subtract_slots(&mut self, req_slots: &mut Vec<i32>) -> bool {
        DecorationCombination::is_possible_static_mut(&mut self.avail_slots, req_slots)
    }

    fn sum(&self) -> (HashMap<String, i32>, Vec<i32>) {
        let mut skills = HashMap::<String, i32>::new();
        let mut slots = Vec::<i32>::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for equip in self.equipments() {
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

        for weapon_slot in &self.weapon_slots {
            if *weapon_slot == 0 {
                continue;
            }

            slots[(weapon_slot - 1) as usize] += 1;
        }

        return (skills, slots);
    }

    pub fn equipments(&self) -> &'a Vec<Box<dyn CalcEquipment<'a>>> {
        &self.equipments
    }
}
