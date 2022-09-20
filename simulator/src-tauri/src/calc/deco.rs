use crate::data::deco::Decoration;

pub struct CalcDeco<'a> {
    pub base: &'a Decoration,
}

impl<'a> CalcDeco<'a> {
    pub fn new(base: &'a Decoration) -> Self {
        Self { base }
    }

    pub fn get_point(slots: &Vec<i32>) -> i32 {
        let mut point = 0;

        for (slot_size_index, count) in slots.iter().enumerate() {
            let slot_level = slot_size_index as i32 + 1;

            let slot_point;

            if slot_level == 4 {
                slot_point = slot_level + 2;
            } else {
                slot_point = slot_level
            }

            point += slot_point * count;
        }

        point
    }
}
