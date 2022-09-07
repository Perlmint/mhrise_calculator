#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use csv::StringRecord;
use deco::Decoration;
use serde::de;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use armor::{BaseArmor, Talisman, TalismanSkill};
use skill::Skill;

use crate::armor::{AnomalyArmor, ArmorSkill, ArmorStat};

mod armor;
mod deco;
mod skill;

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

fn parse_anomaly<'a>(
    filename: &str,
    armors: &'a HashMap<String, BaseArmor>,
    armor_name_dict: &HashMap<&str, &str>,
    skill_name_dict: &HashMap<&str, &str>,
) -> Vec<AnomalyArmor<'a>> {
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

                let mut anomaly_skills = Vec::new();

                for i in (10..record.len()).step_by(2) {
                    let skill_name = &record[i];

                    if skill_name == "" {
                        continue;
                    }

                    let skill_level = to_i32(&record, i + 1);

                    let skill_id = skill_name_dict.get(skill_name).unwrap();

                    let anomaly_skill = ArmorSkill {
                        name: skill_id.to_string(),
                        level: skill_level,
                    };

                    anomaly_skills.push(anomaly_skill);
                }

                let armor_id = *armor_name_dict.get(armor_name).unwrap();
                let armor_info = armors.get(armor_id).unwrap();

                let anomaly_armor = AnomalyArmor {
                    original: &armor_info,
                    stat_diff: stat,
                    slot_diffs: slot_sizes,
                    skill_diffs: anomaly_skills,
                };

                anomaly_armors.push(anomaly_armor);
            }

            anomaly_armors
        }
        Err(_) => Vec::new(),
    }
}

fn parse_talisman(filename: &str, skill_name_dict: &HashMap<&str, &str>) -> Vec<Talisman> {
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
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_count() -> usize {
    let armors_vec = parse_data::<BaseArmor>("data/armor.json");
    let skills_vec = parse_data::<Skill>("data/skill.json");
    let decos_vec = parse_data::<Decoration>("data/deco.json");

    let mut armors = HashMap::<String, BaseArmor>::new();
    let mut skills = HashMap::<String, Skill>::new();

    for armor in armors_vec {
        armors.insert(armor.id.clone(), armor);
    }

    for skill in skills_vec {
        skills.insert(skill.id.clone(), skill);
    }

    let mut armor_name_dict = HashMap::<&str, &str>::new();
    let mut skill_name_dict = HashMap::<&str, &str>::new();

    for pair in &armors {
        let armor = pair.1;

        for lang_name in &armor.names {
            let name = lang_name.1;

            armor_name_dict.insert(name, &armor.id);
        }
    }

    for pair in &skills {
        let skill = pair.1;

        for lang_name in &skill.names {
            let name = lang_name.1;

            skill_name_dict.insert(name, &skill.id);
        }
    }

    let mut qu_armor_filename = "";
    let mut tali_filename = "";

    let anomaly_armors = parse_anomaly(
        qu_armor_filename,
        &armors,
        &armor_name_dict,
        &skill_name_dict,
    );

    let talismans = parse_talisman(tali_filename, &skill_name_dict);

    println!(
        "Anomaly armor count: {}, talisman count: {}",
        anomaly_armors.len(),
        talismans.len()
    );

    return armors.len() + skills.len() + decos_vec.len();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
