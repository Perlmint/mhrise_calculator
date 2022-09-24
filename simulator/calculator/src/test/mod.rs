#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use log::info;

    use crate::{
        calculate_skillset, create_data_manager,
        data::{armor::SexType, deco_combination::DecorationCombination},
    };

    #[test]
    fn it_works() {
        env_logger::init();

        let dm = create_data_manager(
            "./data/armor.json",
            "./data/armor.json",
            "./data/armor.json",
        );

        info!("Armors length: {}", dm.armors.len());

        let mut selected_skills = HashMap::<String, i32>::new();
        let weapon_slots = vec![3, 0, 0];
        let free_slots = vec![0, 0, 0, 0];

        selected_skills.insert("water_attack".to_string(), 5);
        selected_skills.insert("element_exploint".to_string(), 1);

        selected_skills.insert("bow_charge_plus".to_string(), 1);
        selected_skills.insert("spread_shot".to_string(), 3);

        selected_skills.insert("critical_exploit".to_string(), 3);
        selected_skills.insert("chain_crit".to_string(), 1);

        selected_skills.insert("constitution".to_string(), 5);
        selected_skills.insert("stamina_surge".to_string(), 3);

        calculate_skillset(
            weapon_slots,
            selected_skills,
            free_slots,
            SexType::Female,
            &dm,
        );
    }

    #[test]
    fn deco_comb_compare1() {
        assert_eq!(
            DecorationCombination::is_possible_static(&vec![3, 0, 2, 0], &vec![3, 2, 0, 0]),
            true
        );
    }
}
