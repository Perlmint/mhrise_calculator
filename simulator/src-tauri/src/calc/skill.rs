use crate::data::{deco::Decoration, skill::Skill};

pub struct CalcSkill {
    base: &Skill,
    decos: &Vec<Decoration>,
}

impl CalcSkill {
    pub fn new(base: &Skill, decos: Option<Vec<&Decoration>>) -> Self {
        Self { base, decos }
    }

    pub fn calculate_point(skill: &Skill, count: i32, decos: &Vec<Decoration>) -> i32 {
        let mut point = 0;

        if decos.is_none() {
            point += count * 1000;
        } else {
            let max_slot_size = decos.unwrap().last().unwrap().slot_size;
            point += count * max_slot_size;
        }

        point
    }
}
