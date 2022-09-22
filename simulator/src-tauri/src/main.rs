#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub static MAX_ANSWER_LENGTH: i32 = 200;

use csv::StringRecord;
use log::{debug, info};
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;
use std::time::Instant;

use data::armor::{ArmorPart, SexType};
use data::data_manager::DataManager;
use data::deco_combination::DecorationCombinations;
use itertools::iproduct;
use serde::{de, Serialize};
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder};

mod data {
    pub mod armor;
    pub mod data_manager;
    pub mod deco;
    pub mod deco_combination;
    pub mod skill;
}

mod calc {
    pub mod armor;
    pub mod calc_equipment;
    pub mod deco;
    pub mod talisman;
}

mod full_equipments;

mod test;

use crate::calc::armor::CalcArmor;
use crate::calc::calc_equipment::CalcEquipment;
use crate::calc::deco::CalcDeco;
use crate::calc::talisman::CalcTalisman;
use crate::data::armor::{
    AnomalyArmor, ArmorSkill, ArmorStat, BaseArmor, Talisman, TalismanSkill, EMPTY_ARMOR_PREFIX,
};
use crate::data::deco::Decoration;
use crate::data::deco_combination::DecorationCombination;
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

            debug!("Anomaly parsed - count : {}", anomaly_armors.len());

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

            for (index, record) in records.iter().enumerate() {
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

                let talisman =
                    Talisman::new(format!("talisman_{}", index), talisman_skills, slot_sizes);

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

#[derive(Serialize)]
struct CalculateResult {
    full_equipments: Vec<ResultFullEquipments>,
}

#[derive(Serialize)]
struct ResultFullEquipments {
    pub armors: HashMap<String, ResultArmor>,
    pub talisman: ResultTalisman,
    pub deco_combs: Vec<ResultDecorationCombination>,
}

#[derive(Serialize)]
struct ResultArmor {
    pub base_id: String,
    pub is_anomaly: bool,

    pub skills: HashMap<String, i32>,
    pub slots: Vec<i32>,
}

#[derive(Serialize)]
struct ResultTalisman {
    pub skills: HashMap<String, i32>,
    pub slots: Vec<i32>,
}

#[derive(Serialize)]
struct ResultDecorationCombination {
    pub skills: HashMap<String, Vec<i32>>,
    pub slots_sum: Vec<i32>,
}

#[tauri::command]
fn cmd_calculate_skillset(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    mutex_dm: tauri::State<Mutex<DataManager>>,
) -> CalculateSkillsetReturn {
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

    CalculateSkillsetReturn { log, result }
}

type BoxCalcEquipment<'a> = Box<dyn CalcEquipment<'a> + 'a>;

fn calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    sex_type: SexType,
    dm: &'a DataManager,
) -> (String, CalculateResult) {
    let start_time = Instant::now();
    let mut ret = String::from("\n");

    let mut decos_possible = HashMap::<String, Vec<&Decoration>>::new();

    let (no_deco_skills, yes_deco_skills) = dm.get_leftover_skills(&selected_skills);

    for (skill_id, _) in &selected_skills {
        let decos = dm.get_deco_by_skill_id(skill_id);

        if 0 < decos.len() {
            decos_possible.insert(skill_id.clone(), decos);
        }
    }

    info!("Skills with yes deco: {:?}", yes_deco_skills);
    info!("Skills with no deco: {:?}", no_deco_skills);

    let helms = dm.get_parts(ArmorPart::Helm);
    let torsos = dm.get_parts(ArmorPart::Torso);
    let arms = dm.get_parts(ArmorPart::Arm);
    let waists = dm.get_parts(ArmorPart::Waist);
    let feets = dm.get_parts(ArmorPart::Feet);

    let mut base_armors = HashMap::<ArmorPart, Vec<&BaseArmor>>::new();

    base_armors.insert(ArmorPart::Helm, helms);
    base_armors.insert(ArmorPart::Torso, torsos);
    base_armors.insert(ArmorPart::Arm, arms);
    base_armors.insert(ArmorPart::Waist, waists);
    base_armors.insert(ArmorPart::Feet, feets);

    let mut all_armors = HashMap::<ArmorPart, Vec<CalcArmor<'a>>>::new();
    let mut armors_count_before = 0;

    for (part, base_armors) in base_armors.iter() {
        let mut calc_armors = Vec::new();

        for base_armor in base_armors {
            let anomaly_base = dm.get_anomaly_armor(base_armor.id());

            let calc_armor = match anomaly_base {
                Some(anomaly_armor) => {
                    CalcArmor::<'a>::new_anomaly(&anomaly_armor.affected, base_armor)
                }
                None => CalcArmor::<'a>::new_anomaly(base_armor, base_armor),
            };

            calc_armors.push(calc_armor);
            armors_count_before += 1;
        }

        all_armors.insert(part.clone(), calc_armors);
    }

    for (_, armors) in all_armors.iter_mut() {
        *armors = armors
            .iter_mut()
            .filter_map(|armor| {
                if armor.sex_type() == &sex_type || armor.sex_type() == &SexType::All {
                    return Some(armor.clone());
                } else {
                    return None;
                }
            })
            .collect::<Vec<CalcArmor<'a>>>();

        armors.sort_by_key(|armor| {
            armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
        });
    }

    let all_talismans = dm
        .talismans
        .iter()
        .map(|tali| CalcTalisman::<'a>::new(tali))
        .collect::<Vec<CalcTalisman<'a>>>();

    let talisman_count_before = all_talismans.len();

    let mut temp_armors = Vec::new();
    let mut temp_talismans = Vec::new();

    for (part, part_armors) in &all_armors {
        for (i1, armor1) in part_armors.iter().enumerate() {
            let mut is_le = false;

            for i2 in i1 + 1..all_armors.len() {
                let armor2 = &part_armors[i2];

                if armor1.is_le(armor2) {
                    is_le = true;
                    break;
                }
            }

            if is_le == false {
                temp_armors.push((part, armor1));
            }
        }
    }

    for (i1, tali1) in all_talismans.iter().enumerate() {
        let mut is_le = false;

        for i2 in i1 + 1..all_talismans.len() {
            let tali2 = &all_talismans[i2];

            if tali1.is_le(tali2) {
                is_le = true;
                break;
            }
        }

        if is_le == false {
            temp_talismans.push(tali1);
        }
    }

    // TODO: save inferior armors/talismans later in order to give choices to choose for defense stats
    let mut all_armors = HashMap::<ArmorPart, Vec<CalcArmor>>::new();
    let mut armors_count_after = 0;

    for (part, armor) in temp_armors {
        let mut existing = all_armors.get_mut(&part);

        if existing.is_none() {
            all_armors.insert(part.clone(), Vec::<CalcArmor>::new());
            existing = all_armors.get_mut(&part);
        }

        existing.unwrap().push(armor.clone());
        armors_count_after += 1;
    }

    let all_talismans = temp_talismans
        .iter_mut()
        .map(|tali| tali.clone())
        .collect::<Vec<CalcTalisman>>();

    let talisman_count_after = all_talismans.len();

    info!(
        "Armors count before & after: {} -> {}",
        armors_count_before, armors_count_after
    );

    info!(
        "Talisman count before & after: {} -> {}",
        talisman_count_before, talisman_count_after
    );

    let mut all_slot_equips = HashMap::<ArmorPart, HashMap<String, BoxCalcEquipment<'a>>>::new();

    for (part, _) in all_armors.iter() {
        let slot_only_armors = dm.slot_only_armors.get(part).unwrap();
        let mut part_slot_armors = HashMap::<String, BoxCalcEquipment<'a>>::new();

        for (id, armor) in slot_only_armors {
            let calc_armor = CalcArmor::<'a>::new(armor).clone_dyn();

            part_slot_armors.insert(id.clone(), calc_armor);
        }

        all_slot_equips.insert(part.clone(), part_slot_armors);
    }

    all_slot_equips.insert(
        ArmorPart::Talisman,
        dm.slot_only_talismans
            .iter()
            .map(|(id, tali)| (id.clone(), CalcTalisman::new(tali).clone_dyn()))
            .collect::<HashMap<String, BoxCalcEquipment<'a>>>(),
    );

    let mut equips_with_deco_skills = HashMap::<&ArmorPart, Vec<BoxCalcEquipment<'a>>>::new();

    for (part, armors) in &all_armors {
        let mut part_armors = Vec::<BoxCalcEquipment<'a>>::new();

        for armor in armors {
            for (skill_id, _) in &decos_possible {
                if armor.skills().contains_key(skill_id) {
                    part_armors.push(armor.clone_dyn());
                    break;
                }
            }
        }

        equips_with_deco_skills.insert(part, part_armors);
    }

    let mut tali_with_deco_skills = Vec::new();

    for tali in &all_talismans {
        for (skill_id, _) in &decos_possible {
            if tali.skills().contains_key(skill_id) {
                tali_with_deco_skills.push(tali.clone_dyn());
                break;
            }
        }
    }

    equips_with_deco_skills.insert(&ArmorPart::Talisman, tali_with_deco_skills);

    let mut mr_armors = HashMap::<&ArmorPart, Vec<CalcArmor<'a>>>::new();

    for (part, armors) in &all_armors {
        mr_armors.insert(
            part,
            armors
                .iter()
                .filter_map(|armor| {
                    if 7 <= armor.rarity() {
                        return Some(armor.clone());
                    } else {
                        return None;
                    }
                })
                .collect(),
        );
    }

    let mut all_unique_armors = HashMap::<&ArmorPart, Vec<BoxCalcEquipment<'a>>>::new();

    for (part, armors) in &all_armors {
        all_unique_armors.insert(
            part,
            armors
                .iter()
                .filter_map(|armor| {
                    for (skill_id, _) in &no_deco_skills {
                        if armor.skills().contains_key(skill_id) {
                            return Some(armor.clone_dyn());
                        }
                    }

                    None
                })
                .collect::<Vec<BoxCalcEquipment<'a>>>(),
        );

        all_unique_armors
            .get_mut(part)
            .unwrap()
            .push(CalcArmor::<'a>::new(dm.empty_armors.get(&part).unwrap()).clone_dyn());
    }

    all_unique_armors.insert(
        &ArmorPart::Talisman,
        all_talismans
            .iter()
            .filter_map(|tali| {
                for (skill_id, _) in &no_deco_skills {
                    if tali.skills().contains_key(skill_id) {
                        return Some(tali.clone_dyn());
                    }
                }

                None
            })
            .collect(),
    );

    all_unique_armors
        .get_mut(&ArmorPart::Talisman)
        .unwrap()
        .push(CalcTalisman::new(&dm.empty_talisman).clone_dyn());

    for (_, unique_armors) in all_unique_armors.iter_mut() {
        unique_armors.sort_by_key(|armor| {
            armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
        });
    }

    let mut possible_unique_equips = Vec::<Vec<BoxCalcEquipment<'a>>>::new();

    for (helm, torso, arm, waist, feet, tali) in iproduct!(
        all_unique_armors[&ArmorPart::Helm].iter(),
        all_unique_armors[&ArmorPart::Torso].iter(),
        all_unique_armors[&ArmorPart::Arm].iter(),
        all_unique_armors[&ArmorPart::Waist].iter(),
        all_unique_armors[&ArmorPart::Feet].iter(),
        all_unique_armors[&ArmorPart::Talisman].iter()
    ) {
        let equips: Vec<BoxCalcEquipment<'a>> = vec![
            helm.clone_dyn(),
            torso.clone_dyn(),
            arm.clone_dyn(),
            waist.clone_dyn(),
            feet.clone_dyn(),
            tali.clone_dyn(),
        ];

        let full_equip = FullEquipments::<'a>::new(weapon_slots.clone(), equips.clone());
        let (possible_result, possible_combs) = full_equip.get_possible_combs(
            no_deco_skills.clone(),
            &Vec::new(),
            &no_deco_skills,
            &dm.deco_combinations,
        );

        if possible_result {
            debug!(
                "Unique skills possible: {:?}, {:?}",
                possible_combs,
                vec![
                    helm.id(),
                    torso.id(),
                    arm.id(),
                    waist.id(),
                    feet.id(),
                    tali.id()
                ]
            );

            possible_unique_equips.push(equips);
        }
    }

    possible_unique_equips.sort_by_key(|armors| {
        let mut sum = 0;

        for armor in armors {
            sum += armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills);
        }

        Reverse(sum)
    });

    ret.push_str(&format!(
        "Unique armors count: {}, calculation: {:?}\n",
        possible_unique_equips.len(),
        start_time.elapsed()
    ));

    info!(
        "Unique armors count: {}, calculation: {:?}\n",
        possible_unique_equips.len(),
        start_time.elapsed()
    );

    let mut all_parts = Vec::new();

    for possible_unique_vec in &possible_unique_equips {
        debug!(
            "Parts sorted id: {} {} {} {} {} {}",
            possible_unique_vec[0].id(),
            possible_unique_vec[1].id(),
            possible_unique_vec[2].id(),
            possible_unique_vec[3].id(),
            possible_unique_vec[4].id(),
            possible_unique_vec[5].id(),
        );

        let mut parts = possible_unique_vec
            .iter()
            .map(|equipment| {
                let part = equipment.part();

                let mut ret = Vec::<BoxCalcEquipment<'a>>::new();

                if equipment.id().starts_with(EMPTY_ARMOR_PREFIX) {
                    let part_unique_armors = &all_unique_armors[part];
                    let part_deco_armors = &equips_with_deco_skills[part];
                    let part_slot_armors = &all_slot_equips[part]
                        .iter()
                        .filter_map(|(_, armor)| {
                            for unique_armor in part_unique_armors.iter() {
                                if &BaseArmor::get_slot_armor_id(unique_armor.slots()) == armor.id()
                                {
                                    return None;
                                }
                            }

                            for deco_armor in part_deco_armors.iter() {
                                if &BaseArmor::get_slot_armor_id(deco_armor.slots()) == armor.id() {
                                    return None;
                                }
                            }

                            return Some(armor.clone());
                        })
                        .collect::<Vec<BoxCalcEquipment<'a>>>();

                    for armor in part_unique_armors.iter() {
                        ret.push(armor.clone_dyn());
                    }

                    for armor in part_deco_armors.iter() {
                        ret.push(armor.clone_dyn());
                    }

                    for armor in part_slot_armors.iter() {
                        ret.push(armor.clone_dyn());
                    }
                } else {
                    ret.push(equipment.clone_dyn());
                }

                ret.sort_by_key(|armor| {
                    armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
                });

                ret
            })
            .collect::<Vec<Vec<BoxCalcEquipment<'a>>>>();

        parts.sort_by_key(|parts| parts.len());

        let mut total_count = 1;

        for part_armors in &parts {
            total_count *= part_armors.len();
        }

        debug!(
            "Parts size: {} {} {} {} {} {}, total: {}",
            parts[0].len(),
            parts[1].len(),
            parts[2].len(),
            parts[3].len(),
            parts[4].len(),
            parts[5].len(),
            total_count,
        );

        all_parts.push(parts);
    }

    let mut all_unique_parts = HashMap::new();
    let mut all_parts_before_len = 0;

    for parts in &all_parts {
        for (p0, p1, p2, p3, p4, p5) in
            iproduct!(&parts[0], &parts[1], &parts[2], &parts[3], &parts[4], &parts[5])
        {
            all_parts_before_len += 1;

            let equipments = vec![p0, p1, p2, p3, p4, p5];

            let full_equip_id = FullEquipments::get_full_equip_id(&equipments);

            if all_unique_parts.contains_key(&full_equip_id) {
                continue;
            }

            all_unique_parts.insert(full_equip_id, equipments);
        }
    }

    ret.push_str(&format!(
        "Empty armors expand: before count: {}, after count: {}, time: {:?}\n",
        all_parts_before_len,
        all_unique_parts.len(),
        start_time.elapsed()
    ));

    info!(
        "Empty armors expand: before count: {}, after count: {}, time: {:?}\n",
        all_parts_before_len,
        all_unique_parts.len(),
        start_time.elapsed()
    );

    let mut all_loop_tree = BTreeMap::new();
    let mut total_case_count = 0;

    for (_, equipments) in all_unique_parts {
        let mut req_skills = selected_skills.clone();
        let mut req_slots = free_slots.clone();

        let mut real_parts = vec![
            equipments[0].clone(),
            equipments[1].clone(),
            equipments[2].clone(),
            equipments[3].clone(),
            equipments[4].clone(),
            equipments[5].clone(),
        ];

        let real_ids = real_parts
            .iter()
            .map(|part| part.id().clone())
            .collect::<HashSet<String>>();

        let debug_case = vec![
            "rakna_greaves_x",
            "storge_helm",
            "archfiend_armor_baulo",
            "silver_solbraces",
            "lambent_sash",
        ];

        let debug_case = debug_case
            .iter()
            .map(|id| id.to_string())
            .collect::<HashSet<String>>();

        let init_equip = FullEquipments::<'a>::new(weapon_slots.clone(), real_parts.clone());

        for part in real_parts.iter_mut() {
            part.subtract_skills(&mut req_skills);
        }

        let is_debug_case = false;

        /*
        let is_debug_case = debug_case == real_ids;

        if is_debug_case == false {
            continue;
        } else {
            debug!("Debug case reached");
        }
        */

        let (no_deco_skills, single_deco_skills, _) = dm.get_skils_by_deco(&req_skills);

        if no_deco_skills.len() != 0 {
            panic!("This shouldn't happen");
        }

        let single_deco_skills = single_deco_skills
            .iter()
            .map(|(id, (slot_size, count))| (id, *slot_size, *count))
            .collect::<Vec<(&String, i32, i32)>>();

        let single_decos_as_slots = CalcDeco::convert_to_slots(&single_deco_skills);

        for (slot_size_index, count) in single_decos_as_slots.iter().enumerate() {
            req_slots[slot_size_index] += count;
        }

        let mut full_equip = FullEquipments::<'a>::new(weapon_slots.clone(), real_parts.clone());

        let slot_success = full_equip.subtract_slots(&mut req_slots);

        if slot_success == false {
            if is_debug_case {
                debug!(
                    "Debug slots: {:?}, {:?}, {:?}",
                    single_deco_skills, init_equip.avail_slots, req_slots
                );
            }
            continue;
        }

        for (id, _, _) in &single_deco_skills {
            req_skills.remove(*id);
        }

        // This only calculates the number of slots regardless of slot size, just for candidate optimization
        let mut minimum_slot_sum = 0;

        for (skill_id, &level) in &req_skills {
            let mut deco_sum_per_level = dm
                .deco_combinations
                .get(skill_id)
                .unwrap()
                .get(level as usize - 1)
                .unwrap()
                .iter()
                .map(|comb| comb.iter().sum::<i32>())
                .collect::<Vec<i32>>();

            deco_sum_per_level.sort();

            minimum_slot_sum += deco_sum_per_level[0];
        }

        let equip_slot_sum = full_equip.avail_slots.iter().sum::<i32>();

        if equip_slot_sum < minimum_slot_sum {
            if is_debug_case {
                debug!("Debug slots: {}, {}", equip_slot_sum, minimum_slot_sum);
            }
            continue;
        }

        let has_possible_comb = dm
            .deco_combinations
            .has_possible_combs(&req_skills, &full_equip.avail_slots);

        if has_possible_comb == false {
            if is_debug_case {
                debug!(
                    "Debug case no possible combs: {:?}, {:?}",
                    full_equip.avail_slots, req_skills
                );
            }
            continue;
        }

        let total_point = CalcDeco::get_point(&full_equip.avail_slots);

        debug!(
                "Possible candidiate: {:?}\nleft skills: {:?}, slots: {:?}, minimum slots: {}, equip slot sum {}, point: {}",
                real_parts.iter().map(|part| part.id()).collect::<Vec<&String>>(), req_skills, full_equip.avail_slots, minimum_slot_sum,  equip_slot_sum, total_point
            );

        let mut existing = all_loop_tree.get_mut(&Reverse(total_point));

        if existing.is_none() {
            all_loop_tree.insert(
                Reverse(total_point),
                Vec::<(FullEquipments<'a>, HashMap<String, i32>)>::new(),
            );
            existing = all_loop_tree.get_mut(&Reverse(total_point));
        }

        existing.unwrap().push((full_equip, req_skills));
        total_case_count += 1;

        if MAX_ANSWER_LENGTH <= total_case_count {
            debug!(
                "Candidate case count reached {}, breaking",
                MAX_ANSWER_LENGTH
            );
            ret.push_str(&format!(
                "Candidate case count reached {}, breaking\n",
                MAX_ANSWER_LENGTH,
            ));
            break;
        }
    }

    let elapsed_sort = start_time.elapsed();

    ret.push_str(&format!(
        "Candidate cases count: {},\nall_loop_cases sorting elapsed: {:?}\n",
        total_case_count, elapsed_sort
    ));

    debug!(
        "Candidate cases calculated, start calculating: {}\n",
        total_case_count
    );

    let mut total_index = 0;
    let mut answers = Vec::new();

    for (_, case_vec) in &all_loop_tree {
        for (full_equip, req_skills) in case_vec {
            total_index += 1;

            calculate_full_equip(
                dm,
                &req_skills,
                &weapon_slots,
                &full_equip,
                &mut answers,
                &mut total_index,
            );
        }
    }

    info!(
        "All combinations size: {}, answers size: {}",
        total_index,
        answers.len()
    );

    let elapsed_final = start_time.elapsed();

    let mut all_answers_length = 0;

    for (_, deco_combs) in answers.iter() {
        for _ in deco_combs.iter() {
            all_answers_length += 1;
        }
    }

    ret.push_str(&format!(
        "calculate_skillset elapsed: {:?},\nanswers length: {}\n",
        elapsed_final, all_answers_length
    ));
    info!("{}", ret);

    let result_equipments = answers
        .iter()
        .map(|(full_equip, deco_combs)| {
            let result_armors = full_equip
                .equipments()
                .iter()
                .filter_map(|armor| {
                    if armor.part() == &ArmorPart::Talisman {
                        return None;
                    }

                    let armor = armor.as_armor();

                    let result_armor = ResultArmor {
                        base_id: armor.original_id().clone(),
                        is_anomaly: BaseArmor::is_anomaly_armor(armor.id()),
                        skills: armor.skills().clone(),
                        slots: armor.slots().clone(),
                    };

                    Some((armor.part().as_str().to_string(), result_armor))
                })
                .collect::<HashMap<String, ResultArmor>>();

            let result_deco_combs = deco_combs
                .iter()
                .map(|deco_comb| ResultDecorationCombination {
                    skills: deco_comb.combs_per_skill.clone(),
                    slots_sum: deco_comb.sum.clone(),
                })
                .collect::<Vec<ResultDecorationCombination>>();

            let talisman = full_equip.get_by_part(&ArmorPart::Talisman).as_talisman();

            let result_tali = ResultTalisman {
                skills: talisman.skills().clone(),
                slots: talisman.slots().clone(),
            };

            let result_equip = ResultFullEquipments {
                armors: result_armors,
                deco_combs: result_deco_combs,
                talisman: result_tali,
            };

            result_equip
        })
        .collect::<Vec<ResultFullEquipments>>();

    return (
        ret,
        CalculateResult {
            full_equipments: result_equipments,
        },
    );
}

fn compare_failed_slot_armors(slot_armors: &mut HashSet<String>, armor: &CalcArmor) {
    if BaseArmor::is_slot_armor(armor.id()) == false {
        return;
    }

    let mut no_superior_exists = true;
    let mut remove_ids = Vec::new();

    for failed_id in slot_armors.clone() {
        let failed_slots = BaseArmor::parse_slot_armor_id(&failed_id);

        let cmp_result = DecorationCombinations::compare(&failed_slots, &armor.slots());

        if cmp_result == Ordering::Less {
            remove_ids.push(failed_id);
            break;
        } else if cmp_result == Ordering::Greater {
            no_superior_exists = false;
            break;
        }
    }

    for remove_id in remove_ids {
        slot_armors.remove(&remove_id);
    }

    if no_superior_exists {
        slot_armors.insert(armor.id().clone());
    }
}

fn calculate_full_equip<'a>(
    dm: &'a DataManager,
    req_skills: &HashMap<String, i32>,
    weapon_slots: &Vec<i32>,
    full_equip: &FullEquipments<'a>,
    answers: &mut Vec<(FullEquipments<'a>, Vec<DecorationCombination>)>,
    total_index: &mut i32,
) -> i32 {
    let mut possible_deco_combs = dm.deco_combinations.get_possible_combs(&req_skills);
    possible_deco_combs.retain(|comb| comb.is_possible(&full_equip.avail_slots));

    let local_result = possible_deco_combs.len() != 0;

    if local_result == false {
        return 1;
    }

    debug!("Initial slots: {:?}", full_equip.avail_slots);
    debug!("Skill ids: {:?}", full_equip.all_skills);

    for local_answer in &possible_deco_combs {
        debug!("Local answer: {:?}", local_answer);
    }

    debug!(
        "Possible slot combinations: {:?} {:?}",
        possible_deco_combs
            .iter()
            .map(|comb| &comb.combs_per_skill)
            .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
        possible_deco_combs
            .iter()
            .map(|comb| &comb.sum)
            .collect::<Vec<&Vec<i32>>>()
    );

    debug!(
        "Armors ids: ({}), ({}), ({}), ({}), ({}), ({})",
        full_equip.get_by_part(&ArmorPart::Helm).id(),
        full_equip.get_by_part(&ArmorPart::Torso).id(),
        full_equip.get_by_part(&ArmorPart::Arm).id(),
        full_equip.get_by_part(&ArmorPart::Waist).id(),
        full_equip.get_by_part(&ArmorPart::Feet).id(),
        full_equip.get_by_part(&ArmorPart::Talisman).id(),
    );

    debug!(
        "Armors names: ({}), ({}), ({}), ({}), ({})",
        full_equip
            .get_by_part(&ArmorPart::Helm)
            .as_armor()
            .name("ko"),
        full_equip
            .get_by_part(&ArmorPart::Torso)
            .as_armor()
            .name("ko"),
        full_equip
            .get_by_part(&ArmorPart::Arm)
            .as_armor()
            .name("ko"),
        full_equip
            .get_by_part(&ArmorPart::Waist)
            .as_armor()
            .name("ko"),
        full_equip
            .get_by_part(&ArmorPart::Feet)
            .as_armor()
            .name("ko"),
    );

    let mut real_armors = Vec::<Vec<BoxCalcEquipment<'a>>>::new();

    for equipment in full_equip.equipments() {
        let is_slot_equip = BaseArmor::is_slot_armor(equipment.id());

        if is_slot_equip {
            if equipment.part() == &ArmorPart::Talisman {
                let talis_by_slot = dm.talismans_by_slot.get(equipment.id()).unwrap();

                let mut all_real_talis = Vec::new();

                for base_tali in talis_by_slot {
                    let box_tali = CalcTalisman::new(base_tali).clone_dyn();
                    all_real_talis.push(box_tali);
                }

                real_armors.push(all_real_talis);
            } else {
                let armors_by_slot = &dm.armors_by_slot[equipment.part()][equipment.id()];

                let mut all_real_armors = Vec::<BoxCalcEquipment<'a>>::new();

                for base_armor in armors_by_slot {
                    let anomaly_base = dm.get_anomaly_armor(base_armor.id());

                    let calc_armor = match anomaly_base {
                        Some(anomaly_armor) => {
                            CalcArmor::<'a>::new_anomaly(&anomaly_armor.affected, base_armor)
                        }
                        None => CalcArmor::<'a>::new_anomaly(base_armor, base_armor),
                    };

                    let box_armor = calc_armor.clone_dyn();

                    all_real_armors.push(box_armor);
                }

                real_armors.push(all_real_armors);
            }
        } else {
            real_armors.push(vec![equipment.clone_dyn()]);
        }
    }

    let mut answers_equip = Vec::new();

    for (a0, a1, a2, a3, a4, a5) in iproduct!(
        &real_armors[0],
        &real_armors[1],
        &real_armors[2],
        &real_armors[3],
        &real_armors[4],
        &real_armors[5]
    ) {
        let equipments = vec![
            a0.clone(),
            a1.clone(),
            a2.clone(),
            a3.clone(),
            a4.clone(),
            a5.clone(),
        ];

        let final_equip = FullEquipments::<'a>::new(weapon_slots.clone(), equipments);

        answers_equip.push(final_equip);
    }

    for equip in &answers_equip {
        answers.push((equip.clone(), possible_deco_combs.clone()));
    }

    info!(
        "Answers length: {}, total_index: {}\n",
        answers.len(),
        total_index
    );

    return 0;
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
        armors.insert(armor.id().clone(), armor);
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
