#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use deco::Decoration;
use serde::de;
use std::fs::File;
use std::io::BufReader;

use armor::BaseArmor;
use skill::Skill;

mod armor;
mod deco;
mod skill;

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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_count() -> usize {
    let armors = parse_data::<BaseArmor>("data/armor.json");
    let skills = parse_data::<Skill>("data/skill.json");
    let decos = parse_data::<Decoration>("data/deco.json");

    return armors.len() + skills.len() + decos.len();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
