use crate::data::{armor::Talisman, skill::MAX_SLOT_LEVEL};

pub struct CalcTalisman<'a> {
    tali: &'a Talisman,

    slots: Vec<i32>,
}

impl<'a> CalcTalisman<'a> {
    pub fn new(tali: &'a Talisman) -> Self {
        let mut slots = Vec::new();

        for _ in 0..MAX_SLOT_LEVEL {
            slots.push(0);
        }

        for &slot_size in &tali.slot_sizes {
            if slot_size == 0 {
                continue;
            }

            let slot_size_index = slot_size as usize - 1;
            slots[slot_size_index] += 1;
        }

        Self { tali, slots }
    }

    pub fn slots(&self) -> &Vec<i32> {
        &self.slots
    }
}
