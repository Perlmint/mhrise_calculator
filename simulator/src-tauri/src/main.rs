#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs::File;
use std::io::BufReader;

use armor::AnomalyArmor;

mod armor;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_count() -> usize {
    let file = File::open("data/armor.json").unwrap();
    let reader = BufReader::new(file);

    let armors: Vec<AnomalyArmor> = serde_json::from_reader(reader).unwrap();

    return armors.len();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
