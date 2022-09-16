#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{calculate_skillset, create_data_manager, data::armor::SexType};

    #[test]
    fn it_works() {
        let dm = create_data_manager(
            "./data/armor.json",
            "./data/armor.json",
            "./data/armor.json",
        );

        let mut selected_skills = HashMap::<String, i32>::new();
        let free_slots = vec![0, 0, 0, 0];

        selected_skills.insert("constitution".to_string(), 2);
        selected_skills.insert("stamina_surge".to_string(), 2);

        calculate_skillset(
            vec![3, 0, 0],
            selected_skills,
            free_slots,
            SexType::Female,
            &dm,
        );

        println!("Armors length: {}", dm.armors.len());
    }
}
