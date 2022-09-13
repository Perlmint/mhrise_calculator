#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use tauri::StateManager;

    use crate::{calculate_skillset, create_data_manager};

    #[test]
    fn it_works() {
        let dm = create_data_manager(
            "../src/data/armor.json",
            "../src/data/armor.json",
            "../src/data/armor.json",
        );

        let mut selected_skills = HashMap::<String, i32>::new();
        let free_slots = vec![1, 1, 0, 0];

        calculate_skillset(vec![3, 0, 0], selected_skills, free_slots, &dm);

        println!("Armors length: {}", dm.armors.len());
    }
}
