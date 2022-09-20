use std::collections::HashMap;

pub trait CalcEquipment {
    fn skills(&self) -> &HashMap<String, i32>;
    fn slots(&self) -> &Vec<i32>;
}
