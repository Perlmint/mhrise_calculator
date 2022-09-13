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
use serde::de;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder};

mod data {
    pub mod armor;
    pub mod data_manager;
    pub mod deco;
    pub mod skill;
}

mod full_equipments;

mod test;

use crate::data::armor::{AnomalyArmor, ArmorSkill, ArmorStat, BaseArmor, Talisman, TalismanSkill};
use crate::data::deco::Decoration;
use crate::data::skill::Skill;
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
) -> HashMap<String, i32> {
    println!("Start calculating...");

    let dm = mutex_dm.lock().unwrap();

    calculate_skillset(weapon_slots, selected_skills, free_slots, &dm)
}

fn calculate_skillset(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    dm: &DataManager,
) -> HashMap<String, i32> {
    let mut decos_possible = HashMap::<String, Vec<&Decoration>>::new();

    for (skill_id, _) in &selected_skills {
        let decos = dm.get_deco_by_skill_id(skill_id);

        if 0 < decos.len() {
            decos_possible.insert(skill_id.clone(), decos);
        }
    }

    let mut all_combinations = Vec::<FullEquipments>::new();

    let helms = dm.get_parts(ArmorPart::Helm);
    let torsos = dm.get_parts(ArmorPart::Torso);
    let arms = dm.get_parts(ArmorPart::Arm);
    let waists = dm.get_parts(ArmorPart::Waist);
    let feets = dm.get_parts(ArmorPart::Feet);

    let mut mr_helms = Vec::new();
    let mut mr_torsos = Vec::new();
    let mut mr_arms = Vec::new();
    let mut mr_waists = Vec::new();
    let mut mr_feets = Vec::new();

    for helm in helms {
        if 7 <= helm.rarity {
            mr_helms.push(helm);
        }
    }

    for torso in torsos {
        if 7 <= torso.rarity {
            mr_torsos.push(torso);
        }
    }

    for arm in arms {
        if 7 <= arm.rarity {
            mr_arms.push(arm);
        }
    }

    for waist in waists {
        if 7 <= waist.rarity {
            mr_waists.push(waist);
        }
    }

    for feet in feets {
        if 7 <= feet.rarity {
            mr_feets.push(feet);
        }
    }

    println!(
        "Parts size: {} {} {} {} {}, total case: {}",
        mr_helms.len(),
        mr_torsos.len(),
        mr_arms.len(),
        mr_waists.len(),
        mr_feets.len(),
        mr_helms.len() * mr_torsos.len() * mr_arms.len() * mr_waists.len() * mr_feets.len()
    );

    let mut index = 0;

    for helm in &mr_helms {
        if helm.rarity < 7 {
            continue;
        }

        for torso in &mr_torsos {
            if torso.rarity < 7 {
                continue;
            }

            for arm in &mr_arms {
                if arm.rarity < 7 {
                    continue;
                }

                for waist in &mr_waists {
                    if waist.rarity < 7 {
                        continue;
                    }

                    for feet in &mr_feets {
                        if feet.rarity < 7 {
                            continue;
                        }

                        let mut armors = HashMap::<ArmorPart, &BaseArmor>::new();

                        armors.insert(ArmorPart::Helm, helm);
                        armors.insert(ArmorPart::Torso, &torso);
                        armors.insert(ArmorPart::Arm, &arm);
                        armors.insert(ArmorPart::Waist, &waist);
                        armors.insert(ArmorPart::Feet, &feet);

                        let full_equip = FullEquipments::new(weapon_slots.clone(), armors, None);

                        all_combinations.push(full_equip);

                        index += 1;
                    }
                }
                break;
            }
            break;
        }
        break;
    }

    if 0 < dm.talismans.len() {
        let mut talisman_combinations = Vec::<FullEquipments>::new();

        for talisman in &dm.talismans {
            for comb in all_combinations.iter_mut() {
                comb.talisman.replace(&talisman);

                talisman_combinations.push(comb.clone());
            }
        }

        all_combinations = talisman_combinations;
    }

    println!("All combinations size: {}", all_combinations.len());

    for comb in all_combinations {
        let all_combs = comb.is_possible(selected_skills.clone(), &free_slots, &decos_possible);

        for comb in &all_combs {
            println!("Possible comb: {:?}", comb);
        }
    }

    let mut ret = HashMap::<String, i32>::new();
    ret.insert("test1".to_string(), 1);
    ret.insert("test2".to_string(), 3);

    println!("Calculation done");

    return ret;
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
    let dm = create_data_manager(
        "../src/data/armor.json",
        "../src/data/skill.json",
        "../src/data/deco.json",
    );

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
