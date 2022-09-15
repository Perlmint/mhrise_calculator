#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;

use csv::StringRecord;
use data::armor::ArmorPart;
use data::data_manager::DataManager;
use full_equipments::SubSlotSkillCalculator;
use itertools::iproduct;
use serde::de;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder};

mod data {
    pub mod armor;
    pub mod data_manager;
    pub mod deco;
    pub mod deco_combination;
    pub mod skill;
}

mod full_equipments;

mod test;

use crate::data::armor::{AnomalyArmor, ArmorSkill, ArmorStat, BaseArmor, Talisman, TalismanSkill};
use crate::data::deco::Decoration;
use crate::data::deco_combination::DecorationCombinations;
use crate::data::skill::{Skill, MAX_SLOT_LEVEL};
use crate::full_equipments::FullEquipments;

fn to_i32(record: &StringRecord, index: usize) -> i32 {
    return record[index].parse().unwrap();
}

fn parse_data<T>(filename: &str) -> Vec<T>
where
    T: de::DeserializeOwned,
{
    let file = File::open(filename);

    match file {
        Ok(file) => {
            let reader = BufReader::new(file);

            let values: Vec<T> = serde_json::from_reader(reader).unwrap_or(Vec::new());

            values
        }
        Err(_) => Vec::new(),
    }
}

fn parse_anomaly(
    filename: &str,
    armors: &HashMap<String, BaseArmor>,
    armor_name_dict: &HashMap<String, String>,
    skill_name_dict: &HashMap<String, String>,
) -> Vec<AnomalyArmor> {
    let file = File::open(filename);

    match file {
        Ok(file) => {
            let reader = BufReader::new(file);

            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(reader);

            let mut records = Vec::new();

            for result in csv_reader.records() {
                let record = result.unwrap();

                records.push(record);
            }

            let mut anomaly_armors = Vec::new();

            for record in records {
                let armor_name = &record[0];

                let defense = to_i32(&record, 1);

                let fire_res = to_i32(&record, 2);
                let water_res = to_i32(&record, 3);
                let elec_res = to_i32(&record, 4);
                let ice_res = to_i32(&record, 5);
                let dragon_res = to_i32(&record, 6);

                let slot_size1 = to_i32(&record, 7);
                let slot_size2 = to_i32(&record, 8);
                let slot_size3 = to_i32(&record, 9);

                let slot_sizes = vec![slot_size1, slot_size2, slot_size3];

                let stat = ArmorStat {
                    defense,
                    fire_res,
                    water_res,
                    elec_res,
                    ice_res,
                    dragon_res,
                };

                let mut anomaly_skills = HashMap::new();

                for i in (10..record.len()).step_by(2) {
                    let skill_name = &record[i];

                    if skill_name == "" {
                        continue;
                    }

                    let skill_level = to_i32(&record, i + 1);

                    let skill_id = skill_name_dict.get(skill_name).unwrap();

                    let anomaly_skill = ArmorSkill { level: skill_level };

                    anomaly_skills.insert(skill_id.to_string(), anomaly_skill);
                }

                let armor_id = armor_name_dict.get(armor_name).unwrap();
                let armor_info = armors.get(armor_id).unwrap();

                let anomaly_armor =
                    AnomalyArmor::new(armor_info.clone(), stat, slot_sizes, anomaly_skills);

                anomaly_armors.push(anomaly_armor);
            }

            println!("Anomaly parsed - count : {}", anomaly_armors.len());

            anomaly_armors
        }
        Err(_) => Vec::new(),
    }
}

fn parse_talisman(filename: &str, skill_name_dict: &HashMap<String, String>) -> Vec<Talisman> {
    let file = File::open(filename);

    match file {
        Ok(file) => {
            let reader = BufReader::new(file);

            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(reader);

            let mut records = Vec::new();

            for result in csv_reader.records() {
                let record = result.unwrap();

                records.push(record);
            }

            let mut talismans = Vec::new();

            for record in records {
                let skill_name1 = &record[0];
                let skill_level1 = to_i32(&record, 1);
                let skill_name2 = &record[2];
                let skill_level2 = to_i32(&record, 3);

                let slot_size1 = to_i32(&record, 4);
                let slot_size2 = to_i32(&record, 5);
                let slot_size3 = to_i32(&record, 6);

                let slot_sizes = vec![slot_size1, slot_size2, slot_size3];

                let mut talisman_skills = Vec::new();

                if skill_name1 != "" {
                    let skill_id = skill_name_dict.get(skill_name1).unwrap();

                    talisman_skills.push(TalismanSkill {
                        id: skill_id.to_string(),
                        level: skill_level1,
                    });
                }

                if skill_name2 != "" {
                    let skill_id = skill_name_dict.get(skill_name2).unwrap();

                    talisman_skills.push(TalismanSkill {
                        id: skill_id.to_string(),
                        level: skill_level2,
                    });
                }

                let talisman = Talisman {
                    skills: talisman_skills,
                    slot_sizes,
                };

                talismans.push(talisman);
            }

            talismans
        }
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
fn cmd_parse_anomaly(
    filename: &str,
    mutex_dm: tauri::State<Mutex<DataManager>>,
) -> Vec<AnomalyArmor> {
    let mut dm = mutex_dm.lock().unwrap();

    let anomalies = parse_anomaly(
        filename,
        &dm.armors,
        &dm.armor_name_dict,
        &dm.skill_name_dict,
    );

    dm.set_anomalies(anomalies.clone());

    return anomalies;
}

#[tauri::command]
fn cmd_get_skill_names(mutex_dm: tauri::State<Mutex<DataManager>>) -> HashMap<String, Skill> {
    let dm = mutex_dm.lock().unwrap();

    return dm.skills.clone();
}

#[tauri::command]
fn cmd_get_armor_names(mutex_dm: tauri::State<Mutex<DataManager>>) -> HashMap<String, BaseArmor> {
    let dm = mutex_dm.lock().unwrap();

    return dm.armors.clone();
}

#[tauri::command]
fn cmd_calculate_skillset(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    mutex_dm: tauri::State<Mutex<DataManager>>,
) -> Vec<Vec<SubSlotSkillCalculator>> {
    println!("Start calculating...");

    let dm = mutex_dm.lock().unwrap();

    calculate_skillset(weapon_slots, selected_skills, free_slots, &dm)
}

fn calculate_skillset(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    dm: &DataManager,
) -> Vec<Vec<SubSlotSkillCalculator>> {
    let mut decos_possible = HashMap::<String, Vec<&Decoration>>::new();
    let mut no_deco_skills = HashMap::<String, i32>::new();

    for (skill_id, level) in &selected_skills {
        let decos = dm.get_deco_by_skill_id(skill_id);

        if 0 < decos.len() {
            decos_possible.insert(skill_id.clone(), decos);
        } else {
            no_deco_skills.insert(skill_id.clone(), *level);
        }
    }

    println!("Skills with no deco: {:?}", no_deco_skills);

    let mut helms = dm.get_parts_clone(ArmorPart::Helm);
    let mut torsos = dm.get_parts_clone(ArmorPart::Torso);
    let mut arms = dm.get_parts_clone(ArmorPart::Arm);
    let mut waists = dm.get_parts_clone(ArmorPart::Waist);
    let mut feets = dm.get_parts_clone(ArmorPart::Feet);

    let mut all_armors = HashMap::new();

    all_armors.insert(ArmorPart::Helm, &mut helms);
    all_armors.insert(ArmorPart::Torso, &mut torsos);
    all_armors.insert(ArmorPart::Arm, &mut arms);
    all_armors.insert(ArmorPart::Waist, &mut waists);
    all_armors.insert(ArmorPart::Feet, &mut feets);

    for (_, armors) in all_armors.iter_mut() {
        armors.sort_by_key(|armor| std::cmp::Reverse(armor.rarity));
        armors.sort_by(|a1, a2| {
            let slots1 = &a1.slots;
            let slots2 = &a2.slots;

            for i in (0..slots1.len()).rev() {
                let slot_count1 = slots1[i];
                let slot_count2 = slots2[i];

                if slot_count1 != slot_count2 {
                    return slot_count2.cmp(&slot_count1);
                }
            }

            return std::cmp::Ordering::Equal;
        });
    }

    let mut all_unique_armors = HashMap::<ArmorPart, Vec<BaseArmor>>::new();

    for (part, armors) in &all_armors {
        all_unique_armors.insert(
            part.clone(),
            armors
                .iter()
                .filter_map(|armor| {
                    for (skill_id, _) in &no_deco_skills {
                        if armor.skills.contains_key(skill_id) {
                            return Some(armor.clone());
                        }
                    }

                    return None;
                })
                .collect(),
        );

        all_unique_armors
            .get_mut(part)
            .unwrap()
            .push(BaseArmor::create_empty(part.clone()));
    }

    let mut possible_unique_armors = Vec::new();

    for (helm, torso, arm, waist, feet) in iproduct!(
        all_unique_armors[&ArmorPart::Helm].iter(),
        all_unique_armors[&ArmorPart::Torso].iter(),
        all_unique_armors[&ArmorPart::Arm].iter(),
        all_unique_armors[&ArmorPart::Waist].iter(),
        all_unique_armors[&ArmorPart::Feet].iter()
    ) {
        let mut armors = HashMap::new();
        armors.insert(ArmorPart::Helm, helm);
        armors.insert(ArmorPart::Torso, torso);
        armors.insert(ArmorPart::Arm, arm);
        armors.insert(ArmorPart::Waist, waist);
        armors.insert(ArmorPart::Feet, feet);

        let full_equip = FullEquipments::new(weapon_slots.clone(), armors.clone(), None);
        let possible_combs =
            full_equip.is_possible(no_deco_skills.clone(), &free_slots, &decos_possible);

        if 0 < possible_combs.len() {
            println!(
                "Unique skills possible: {:?}, {:?}",
                possible_combs,
                vec![
                    helm.id.clone(),
                    torso.id.clone(),
                    arm.id.clone(),
                    waist.id.clone(),
                    feet.id.clone()
                ]
            );

            possible_unique_armors.push(armors);
        }
    }

    let get_not_empty_count = |armors: &HashMap<ArmorPart, &BaseArmor>| -> i32 {
        let mut count = 0;

        for (_, armor) in armors {
            if armor.id.starts_with("_empty_") == false {
                count += 1;
            }
        }

        count
    };

    possible_unique_armors.sort_by(|a1, a2| {
        let empty_count1 = get_not_empty_count(a1);
        let empty_count2 = get_not_empty_count(a2);

        empty_count2.cmp(&empty_count1)
    });

    let mut mr_armors = HashMap::<ArmorPart, Vec<BaseArmor>>::new();

    for (part, armors) in &all_armors {
        mr_armors.insert(
            part.clone(),
            armors
                .iter()
                .filter_map(|armor| {
                    if 7 <= armor.rarity {
                        return Some(armor.clone());
                    } else {
                        return None;
                    }
                })
                .collect(),
        );
    }

    let mut total_index = 0;

    let mut answers = Vec::new();

    'all_cases: for armors in &possible_unique_armors {
        let mut helms = vec![armors[&ArmorPart::Helm].clone()];
        let mut torsos = vec![armors[&ArmorPart::Torso].clone()];
        let mut arms = vec![armors[&ArmorPart::Arm].clone()];
        let mut waists = vec![armors[&ArmorPart::Waist].clone()];
        let mut feets = vec![armors[&ArmorPart::Feet].clone()];

        let mut local_armors = HashMap::<ArmorPart, &mut Vec<BaseArmor>>::new();
        local_armors.insert(ArmorPart::Helm, &mut helms);
        local_armors.insert(ArmorPart::Torso, &mut torsos);
        local_armors.insert(ArmorPart::Arm, &mut arms);
        local_armors.insert(ArmorPart::Waist, &mut waists);
        local_armors.insert(ArmorPart::Feet, &mut feets);

        println!(
            "Base armors ids: {:?}",
            &local_armors
                .iter()
                .map(|(_, armor)| armor[0].id.clone())
                .collect::<Vec<String>>()
        );

        let modify_empty_parts = |part: &ArmorPart, part_armors: &mut Vec<BaseArmor>| {
            if part_armors.len() == 1 && part_armors[0].id.starts_with("_empty_") {
                part_armors.clear();
                *part_armors = mr_armors[part].clone();
            }
        };

        for (part, part_armors) in local_armors.iter_mut() {
            modify_empty_parts(part, part_armors);
        }

        let mut local_index = 0;
        let mut total_count = 1;

        for (_, part_armors) in &local_armors {
            total_count *= part_armors.len();
        }

        let mut parts = vec![helms, torsos, arms, waists, feets];

        parts.sort_by_key(|p| p.len());

        let ten_percent = total_count / 10;

        println!(
            "Parts size: {} {} {} {} {}, total: {}, 10%: {}",
            parts[0].len(),
            parts[1].len(),
            parts[2].len(),
            parts[3].len(),
            parts[4].len(),
            total_count,
            ten_percent
        );

        let subtracted_armors = |parts: &BaseArmor,
                                 req_skills: &mut HashMap<String, i32>,
                                 req_slots: &mut Vec<i32>|
         -> (BaseArmor, HashMap<String, i32>) {
            let mut parts = parts.clone();
            let diff_skills = parts.subtract_skills(req_skills, req_slots);

            return (parts, diff_skills);
        };

        for p1 in &parts[0] {
            let mut req_skills = selected_skills.clone();
            let mut free_slots = free_slots.clone();
            let (p1, s1) = &subtracted_armors(p1, &mut req_skills, &mut free_slots);

            for p2 in &parts[1] {
                let mut req_skills = req_skills.clone();
                let mut free_slots = free_slots.clone();
                let (p2, s2) = &subtracted_armors(p2, &mut req_skills, &mut free_slots);

                for p3 in &parts[2] {
                    let mut req_skills = req_skills.clone();
                    let mut free_slots = free_slots.clone();
                    let (p3, s3) = &subtracted_armors(p3, &mut req_skills, &mut free_slots);

                    for p4 in &parts[3] {
                        let mut req_skills = req_skills.clone();
                        let mut free_slots = free_slots.clone();
                        let (p4, s4) = &subtracted_armors(p4, &mut req_skills, &mut free_slots);

                        'final_armor: for p5 in &parts[4] {
                            let mut req_skills = req_skills.clone();
                            let mut free_slots = free_slots.clone();
                            let (p5, s5) = &subtracted_armors(p5, &mut req_skills, &mut free_slots);

                            for slot_count in &free_slots {
                                if 0 < *slot_count {
                                    continue 'final_armor;
                                }
                            }

                            let mut armors = HashMap::<ArmorPart, &BaseArmor>::new();

                            armors.insert(p1.part.clone(), p1);
                            armors.insert(p2.part.clone(), p2);
                            armors.insert(p3.part.clone(), p3);
                            armors.insert(p4.part.clone(), p4);
                            armors.insert(p5.part.clone(), p5);

                            let full_equip =
                                FullEquipments::new(weapon_slots.clone(), armors, None);

                            let all_possible_slot_coms =
                                dm.deco_combinations.get_possible_combs(&req_skills);

                            let armor_possible_slot_combs = all_possible_slot_coms
                                .iter()
                                .filter_map(|possible_comb| {
                                    let mut is_available = true;

                                    for i in 0..MAX_SLOT_LEVEL {
                                        let i = i as usize;

                                        let armor_slot_count = full_equip.avail_slots[i];
                                        let req_slot_count = possible_comb[i];

                                        if armor_slot_count < req_slot_count {
                                            is_available = false;
                                            break;
                                        }
                                    }

                                    if is_available {
                                        Some(possible_comb.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<Vec<i32>>>();

                            if 0 < armor_possible_slot_combs.len() {
                                let subtracts = vec![s1, s2, s3, s4, s5];

                                let mut subtracted_skills = HashMap::<String, i32>::new();

                                for s in subtracts {
                                    for (s_id, s_level) in s {
                                        let existing = subtracted_skills.get_mut(s_id);

                                        match existing {
                                            Some(prev_level) => *prev_level += s_level,
                                            None => {
                                                subtracted_skills.insert(s_id.clone(), *s_level);
                                            }
                                        }
                                    }
                                }

                                println!("Initial slots: {:?}", full_equip.avail_slots);
                                println!("All skills: {:?}", full_equip.all_skills);
                                println!("Subracted skills: {:?}", subtracted_skills);
                                println!("Requested skills: {:?}", selected_skills);
                                println!(
                                    "Decos possible: {:?}",
                                    &decos_possible
                                        .iter()
                                        .map(|deco| deco.0)
                                        .collect::<Vec<&String>>()
                                );
                                println!(
                                    "Possible slot combinations: {:?}",
                                    all_possible_slot_coms
                                );

                                for comb in &armor_possible_slot_combs {
                                    println!("Possible comb: {:?}", comb);
                                }

                                println!("No decos {:?}", no_deco_skills);
                                println!(
                                    "Armors ids: {:?}",
                                    &full_equip
                                        .armors
                                        .into_iter()
                                        .map(|(_, armor)| armor.id.clone())
                                        .collect::<Vec<String>>()
                                );

                                answers.push(armor_possible_slot_combs);

                                println!("Answers length: {}", answers.len());
                                println!();
                            }

                            if 200 <= answers.len() {
                                println!("Iteration size too large, breaking at 200");
                                break 'all_cases;
                            }

                            total_index += 1;
                            local_index += 1;

                            // if local_index % ten_percent == 0 {
                            //     println!("{}% passed", 10 * local_index / ten_percent);
                            // }
                        }
                    }
                }
            }
        }
    }

    println!("All combinations size: {}", total_index + 1);

    return Vec::new();
}

fn create_data_manager(
    armors_filename: &str,
    skills_filename: &str,
    decos_filename: &str,
) -> DataManager {
    let armors_vec = parse_data::<BaseArmor>(armors_filename);
    let skills_vec = parse_data::<Skill>(skills_filename);
    let decos_vec = parse_data::<Decoration>(decos_filename);

    let mut armors = HashMap::<String, BaseArmor>::new();
    let mut skills = HashMap::<String, Skill>::new();
    let mut decos = HashMap::<String, Decoration>::new();

    for armor in armors_vec {
        armors.insert(armor.id.clone(), armor);
    }

    for skill in skills_vec {
        skills.insert(skill.id.clone(), skill);
    }

    for deco in decos_vec {
        decos.insert(deco.id.clone(), deco);
    }

    let dm = DataManager::new(armors, skills, decos);

    dm
}

fn main() {
    let dm = create_data_manager("./data/armor.json", "./data/skill.json", "./data/deco.json");

    println!(
        "Anomaly armor count: {}, talisman count: {}",
        dm.anomaly_armors.len(),
        dm.talismans.len()
    );

    let anomaly_submenu = CustomMenuItem::new("anomaly_crafting".to_string(), "Anomaly Crafting");

    let data_submenu = Submenu::new("Data", Menu::new().add_item(anomaly_submenu));

    let menu = Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_submenu(data_submenu);

    tauri::Builder::default()
        .manage(Mutex::new(dm))
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "anomaly_crafting" => {
                let handle = event.window().app_handle();

                let existing = event.window().get_window("anomaly_list");

                if existing.is_none() {
                    WindowBuilder::new(
                        &handle,
                        "anomaly_list".to_string(),
                        tauri::WindowUrl::App("anomaly_list.html".into()),
                    )
                    .title("Anomaly Crafting List")
                    .build()
                    .unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            cmd_parse_anomaly,
            cmd_get_skill_names,
            cmd_get_armor_names,
            cmd_calculate_skillset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
