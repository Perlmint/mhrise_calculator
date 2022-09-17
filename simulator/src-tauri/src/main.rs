#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use csv::StringRecord;
use data::armor::{ArmorPart, SexType};
use data::data_manager::DataManager;
use data::deco_combination::DecorationCombinations;
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

mod calc {
    pub mod armor;
    pub mod deco;
}

mod full_equipments;

mod test;

use crate::calc::armor::CalcArmor;
use crate::data::armor::{
    AnomalyArmor, ArmorSkill, ArmorStat, BaseArmor, Talisman, TalismanSkill, EMPTY_ARMOR_PREFIX,
};
use crate::data::deco::Decoration;
use crate::data::deco_combination::DecorationCombination;
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
fn cmd_calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    mutex_dm: tauri::State<'a, Mutex<DataManager>>,
) -> String {
    println!("Start calculating...");

    let dm = mutex_dm.lock().unwrap();

    // TODO get sex_type as input
    calculate_skillset(
        weapon_slots,
        selected_skills,
        free_slots,
        SexType::Female,
        &dm,
    )
}

fn sort_by_rarity(armor: &CalcArmor) -> std::cmp::Reverse<i32> {
    std::cmp::Reverse(armor.rarity())
}

fn sort_by_slot(a1: &CalcArmor, a2: &CalcArmor) -> std::cmp::Ordering {
    let slots1 = &a1.slots();
    let slots2 = &a2.slots();

    return DecorationCombinations::compare(slots2, slots1);
}

fn calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    sex_type: SexType,
    dm: &'a DataManager,
) -> String {
    type FullArmors<'a> = HashMap<ArmorPart, CalcArmor<'a>>;

    let start_time = Instant::now();

    let mut decos_possible = HashMap::<String, Vec<&Decoration>>::new();
    let mut yes_deco_skills = HashMap::<String, i32>::new();
    let mut no_deco_skills = HashMap::<String, i32>::new();

    for (skill_id, level) in &selected_skills {
        let decos = dm.get_deco_by_skill_id(skill_id);

        if 0 < decos.len() {
            decos_possible.insert(skill_id.clone(), decos);
            yes_deco_skills.insert(skill_id.clone(), *level);
        } else {
            no_deco_skills.insert(skill_id.clone(), *level);
        }
    }

    println!("Skills with yes deco: {:?}", yes_deco_skills);
    println!("Skills with no deco: {:?}", no_deco_skills);

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

    for (part, base_armors) in base_armors.iter() {
        let mut calc_armors = Vec::new();

        for base_armor in base_armors {
            let calc_armor = CalcArmor::<'a>::new(base_armor);
            calc_armors.push(calc_armor);
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

        for part_armor in armors.iter_mut() {
            part_armor.calculate_point(&decos_possible, &yes_deco_skills, &no_deco_skills);
        }

        armors.sort_by_key(|armor| armor.point());
    }

    let mut all_slot_armors = HashMap::<ArmorPart, HashMap<String, CalcArmor<'a>>>::new();

    for (part, _) in all_armors.iter() {
        let slot_only_armors = dm.slot_only_armors.get(part).unwrap();
        let mut part_slot_armors = HashMap::<String, CalcArmor<'a>>::new();

        for (id, armor) in slot_only_armors {
            let mut calc_armor = CalcArmor::<'a>::new(armor);
            calc_armor.calculate_point(&decos_possible, &yes_deco_skills, &no_deco_skills);

            part_slot_armors.insert(id.clone(), calc_armor);
        }

        all_slot_armors.insert(part.clone(), part_slot_armors);
    }

    let mut all_unique_armors = HashMap::<&ArmorPart, Vec<CalcArmor<'a>>>::new();

    for (part, armors) in &all_armors {
        all_unique_armors.insert(
            part,
            armors
                .iter()
                .filter_map(|armor| {
                    for (skill_id, _) in &no_deco_skills {
                        if armor.skills().contains_key(skill_id) {
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
            .push(CalcArmor::<'a>::new(dm.empty_armors.get(&part).unwrap()));
    }

    let mut possible_unique_armors = Vec::new();

    for (helm, torso, arm, waist, feet) in iproduct!(
        all_unique_armors[&ArmorPart::Helm].iter(),
        all_unique_armors[&ArmorPart::Torso].iter(),
        all_unique_armors[&ArmorPart::Arm].iter(),
        all_unique_armors[&ArmorPart::Waist].iter(),
        all_unique_armors[&ArmorPart::Feet].iter()
    ) {
        let mut armors = FullArmors::new();
        armors.insert(helm.part(), helm.clone());
        armors.insert(torso.part(), torso.clone());
        armors.insert(arm.part(), arm.clone());
        armors.insert(waist.part(), waist.clone());
        armors.insert(feet.part(), feet.clone());

        let full_equip = FullEquipments::new(weapon_slots.clone(), armors.clone(), None);
        let possible_combs =
            full_equip.is_possible(no_deco_skills.clone(), &free_slots, &decos_possible);

        if 0 < possible_combs.len() {
            println!(
                "Unique skills possible: {:?}, {:?}",
                possible_combs,
                vec![helm.id(), torso.id(), arm.id(), waist.id(), feet.id()]
            );

            possible_unique_armors.push(armors);
        }
    }

    let get_not_empty_count = |armors: &HashMap<ArmorPart, CalcArmor<'a>>| -> i32 {
        let mut count = 0;

        for (_, armor) in armors {
            if armor.id().starts_with(EMPTY_ARMOR_PREFIX) == false {
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

    let mut armors_with_deco_skills = HashMap::<&ArmorPart, Vec<CalcArmor<'a>>>::new();

    for (part, armors) in &all_armors {
        let mut part_armors = Vec::<CalcArmor<'a>>::new();

        for armor in armors {
            for (skill_id, _) in &decos_possible {
                if armor.skills().contains_key(skill_id) {
                    part_armors.push(armor.clone());
                    break;
                }
            }
        }

        armors_with_deco_skills.insert(part, part_armors);
    }

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

    let mut total_index = 0;

    let mut answers = Vec::new();

    'all_cases: for armors in &possible_unique_armors {
        let mut helms = vec![armors[&ArmorPart::Helm].clone()];
        let mut torsos = vec![armors[&ArmorPart::Torso].clone()];
        let mut arms = vec![armors[&ArmorPart::Arm].clone()];
        let mut waists = vec![armors[&ArmorPart::Waist].clone()];
        let mut feets = vec![armors[&ArmorPart::Feet].clone()];

        let mut local_armors = HashMap::<&ArmorPart, &mut Vec<CalcArmor<'a>>>::new();
        local_armors.insert(&ArmorPart::Helm, &mut helms);
        local_armors.insert(&ArmorPart::Torso, &mut torsos);
        local_armors.insert(&ArmorPart::Arm, &mut arms);
        local_armors.insert(&ArmorPart::Waist, &mut waists);
        local_armors.insert(&ArmorPart::Feet, &mut feets);

        println!(
            "Base armors ids: {:?}",
            &local_armors
                .iter()
                .map(|(_, armor)| armor[0].id())
                .collect::<Vec<&String>>()
        );

        let modify_empty_parts = |part: &ArmorPart, part_armors: &mut Vec<CalcArmor<'a>>| {
            if part_armors.len() == 1 && part_armors[0].id().starts_with(EMPTY_ARMOR_PREFIX) {
                part_armors.clear();

                let mut deco_armors = armors_with_deco_skills[part].clone();
                let mut part_slot_armors = all_slot_armors[part]
                    .values()
                    .map(|armor| armor.clone())
                    .collect::<Vec<CalcArmor<'a>>>()
                    .clone();

                part_armors.append(&mut deco_armors);
                part_armors.append(&mut part_slot_armors);

                part_armors.sort_by_key(|armor| armor.point());
            }
        };

        for (part, part_armors) in local_armors.iter_mut() {
            modify_empty_parts(part, part_armors);
        }

        let mut total_count = 1;

        for (_, part_armors) in &local_armors {
            total_count *= part_armors.len();
        }

        let mut parts = vec![helms, torsos, arms, waists, feets];

        parts.sort_by_key(|p| p.len());

        println!(
            "Parts size: {} {} {} {} {}, total: {}",
            parts[0].len(),
            parts[1].len(),
            parts[2].len(),
            parts[3].len(),
            parts[4].len(),
            total_count,
        );
        println!();

        let subtracted_armors = |parts: &CalcArmor<'a>,
                                 req_skills: &mut HashMap<String, i32>,
                                 req_slots: &mut Vec<i32>|
         -> (CalcArmor<'a>, HashMap<String, i32>) {
            let mut parts: CalcArmor<'a> = parts.clone();
            let diff_skills = parts.subtract_skills(req_skills, req_slots);

            return (parts, diff_skills);
        };

        for p1 in &parts[0] {
            let mut req_skills = selected_skills.clone();
            let mut free_slots = free_slots.clone();
            let (p1, s1) = subtracted_armors(p1, &mut req_skills, &mut free_slots);

            for p2 in &parts[1] {
                let mut req_skills = req_skills.clone();
                let mut free_slots = free_slots.clone();
                let (p2, s2) = subtracted_armors(p2, &mut req_skills, &mut free_slots);

                for p3 in &parts[2] {
                    let mut req_skills = req_skills.clone();
                    let mut free_slots = free_slots.clone();
                    let (p3, s3) = subtracted_armors(p3, &mut req_skills, &mut free_slots);

                    for p4 in &parts[3] {
                        let mut req_skills = req_skills.clone();
                        let mut free_slots = free_slots.clone();
                        let (p4, s4) = subtracted_armors(p4, &mut req_skills, &mut free_slots);

                        let mut failed_slot_armors = HashSet::<String>::new();

                        let compare_failed_slot_armors =
                            |slot_armors: &mut HashSet<String>, armor: &CalcArmor<'a>| {
                                if BaseArmor::is_slot_armor(armor.id()) == false {
                                    return;
                                }

                                let mut no_superior_exists = true;
                                let mut remove_ids = Vec::new();

                                for failed_id in slot_armors.clone() {
                                    let failed_slots = BaseArmor::parse_slot_armor_id(&failed_id);

                                    let cmp_result = DecorationCombinations::compare(
                                        &failed_slots,
                                        &armor.slots(),
                                    );

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
                            };

                        'final_armor: for p5 in &parts[4] {
                            total_index += 1;

                            for failed_id in failed_slot_armors.clone() {
                                if DecorationCombinations::compare(
                                    &BaseArmor::parse_slot_armor_id(&failed_id),
                                    &p5.slots(),
                                ) != Ordering::Less
                                {
                                    continue;
                                }
                            }

                            let mut req_skills = req_skills.clone();
                            let mut free_slots = free_slots.clone();
                            let (p5, s5) = subtracted_armors(p5, &mut req_skills, &mut free_slots);

                            for slot_count in &free_slots {
                                if 0 < *slot_count {
                                    compare_failed_slot_armors(&mut failed_slot_armors, &p5);

                                    continue 'final_armor;
                                }
                            }

                            let mut armors = FullArmors::new();

                            armors.insert(p1.part(), p1.clone());
                            armors.insert(p2.part(), p2.clone());
                            armors.insert(p3.part(), p3.clone());
                            armors.insert(p4.part(), p4.clone());
                            armors.insert(p5.part(), p5.clone());

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
                                        let req_slot_count = possible_comb.sum[i];

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
                                .collect::<Vec<DecorationCombination>>();

                            if armor_possible_slot_combs.len() == 0 {
                                compare_failed_slot_armors(&mut failed_slot_armors, &p5);
                                continue;
                            }

                            let subtracts = vec![&s1, &s2, &s3, &s4, &s5];

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

                            /*
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
                                "Possible slot combinations: {:?} {:?}",
                                all_possible_slot_coms
                                    .iter()
                                    .map(|comb| comb.combs_per_skill.clone())
                                    .collect::<Vec<HashMap<String, Vec<i32>>>>(),
                                all_possible_slot_coms
                                    .iter()
                                    .map(|comb| comb.sum.clone())
                                    .collect::<Vec<Vec<i32>>>()
                            );

                            for comb in &armor_possible_slot_combs {
                                println!("Possible comb: {:?}", comb);
                            }

                            println!("No decos {:?}", no_deco_skills);
                            */

                            println!(
                                "Armors ids: ({}), ({}), ({}), ({}), ({})",
                                full_equip.armors[&ArmorPart::Helm].id(),
                                full_equip.armors[&ArmorPart::Torso].id(),
                                full_equip.armors[&ArmorPart::Arm].id(),
                                full_equip.armors[&ArmorPart::Waist].id(),
                                full_equip.armors[&ArmorPart::Feet].id(),
                            );

                            let mut real_armors = Vec::<Vec<CalcArmor<'a>>>::new();

                            for (part, armor) in full_equip.armors {
                                if BaseArmor::is_slot_armor(armor.id()) {
                                    let all_real_armors = dm
                                        .armors_by_slot
                                        .get(&part)
                                        .unwrap()
                                        .get(armor.id())
                                        .unwrap()
                                        .iter()
                                        .map(|base_armor| CalcArmor::<'a>::new(base_armor))
                                        .collect::<Vec<CalcArmor<'a>>>();

                                    real_armors.push(all_real_armors.clone());
                                } else {
                                    real_armors.push(vec![armor]);
                                }
                            }

                            let mut answers_equip = Vec::new();

                            for (a1, a2, a3, a4, a5) in iproduct!(
                                &real_armors[0],
                                &real_armors[1],
                                &real_armors[2],
                                &real_armors[3],
                                &real_armors[4]
                            ) {
                                let mut armors = HashMap::new();
                                armors.insert(a1.part(), a1.clone());
                                armors.insert(a2.part(), a2.clone());
                                armors.insert(a3.part(), a3.clone());
                                armors.insert(a4.part(), a4.clone());
                                armors.insert(a5.part(), a5.clone());

                                let final_equip =
                                    FullEquipments::new(weapon_slots.clone(), armors, None);

                                answers_equip.push(final_equip);
                            }

                            for (equip, slot_comb) in
                                iproduct!(answers_equip, armor_possible_slot_combs)
                            {
                                answers.push((equip, slot_comb));
                            }

                            println!(
                                "Answers length: {}, total_index: {}",
                                answers.len(),
                                total_index
                            );
                            println!();

                            if 200 <= answers.len() {
                                println!("Iteration size too large, breaking at 200");
                                break 'all_cases;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("All combinations size: {}", total_index + 1);

    let elapsed = start_time.elapsed();

    let ret = format!("calculate_skillset elapsed: {:?}", elapsed);
    println!("{}", ret);

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
