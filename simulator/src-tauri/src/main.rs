#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use data::armor::SexType;
use log::debug;
use mhr_calculator::{
    data::{
        armor::{AnomalyArmor, BaseArmor, Talisman},
        data_manager::DataManager,
        skill::Skill,
    },
    *,
};
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex};
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder};

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
fn cmd_parse_talisman(filename: &str, mutex_dm: tauri::State<Mutex<DataManager>>) -> Vec<Talisman> {
    let mut dm = mutex_dm.lock().unwrap();

    let talismans = parse_talisman(filename, &dm.skill_name_dict);

    dm.set_talismans(talismans.clone());

    return talismans;
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

#[derive(Serialize)]
struct CalculateSkillsetReturn {
    log: String,
    result: CalculateResult,
}

#[tauri::command]
async fn cmd_calculate_skillset(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    mutex_dm: tauri::State<'_, Mutex<DataManager>>,
) -> Result<CalculateSkillsetReturn, ()> {
    debug!("Start calculating...");

    let dm = mutex_dm.lock().unwrap();

    // TODO get sex_type as input
    let (log, result) = calculate_skillset(
        weapon_slots,
        selected_skills,
        free_slots,
        SexType::Female,
        &dm,
    );

    Ok(CalculateSkillsetReturn { log, result })
}

fn main() {
    env_logger::init();

    let dm = create_data_manager("./data/armor.json", "./data/skill.json", "./data/deco.json");

    debug!(
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
            cmd_parse_talisman,
            cmd_get_skill_names,
            cmd_get_armor_names,
            cmd_calculate_skillset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
