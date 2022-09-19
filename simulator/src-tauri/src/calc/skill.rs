use crate::data::{deco::Decoration, skill::Skill};

pub struct CalcSkill<'a> {
    base: &'a Skill,
    decos: Option<Vec<&'a Decoration>>,
}

impl<'a> CalcSkill<'a> {
    pub fn new(base: &'a Skill, decos: Option<Vec<&'a Decoration>>) -> Self {
        Self { base, decos }
    }

    pub fn calculate_point(skill: &Skill, count: i32, decos: Option<Vec<&Decoration>>) -> i32 {
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
