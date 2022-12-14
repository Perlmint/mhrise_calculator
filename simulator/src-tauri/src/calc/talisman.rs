use std::collections::HashMap;

use crate::data::{
    armor::{ArmorPart, Talisman},
    skill::MAX_SLOT_LEVEL,
};

use super::{armor::CalcArmor, calc_equipment::CalcEquipment};

#[derive(Debug, Clone)]
pub struct CalcTalisman<'a> {
    tali: &'a Talisman,

    slots: Vec<i32>,
    skills: HashMap<String, i32>,
}

impl<'a> CalcTalisman<'a> {
    pub fn new(tali: &'a Talisman) -> Self {
        let slots = CalcArmor::convert_from_base_slots(&tali.slot_sizes);

        let mut skills = HashMap::new();

        for tali_skill in &tali.skills {
            skills.insert(tali_skill.id.clone(), tali_skill.level);
        }

        Self {
            tali,
            slots,
            skills,
        }
    }
}

impl<'a> CalcEquipment<'a> for CalcTalisman<'a> {
    fn id(&self) -> &String {
        &self.tali.id()
    }

    fn skills(&self) -> &HashMap<String, i32> {
        &self.skills
    }

    fn mut_skills(&mut self) -> &mut HashMap<String, i32> {
        &mut self.skills
    }

    fn slots(&self) -> &Vec<i32> {
        &self.slots
    }

    fn part(&self) -> &ArmorPart {
        &ArmorPart::Talisman
    }

    fn clone_dyn(&self) -> Box<dyn CalcEquipment<'a> + 'a> {
        Box::new(self.clone())
    }

    fn as_armor(&self) -> &CalcArmor<'a> {
        panic!("This is not armor");
    }

    fn as_talisman(&self) -> &CalcTalisman<'a> {
        self
    }
}
