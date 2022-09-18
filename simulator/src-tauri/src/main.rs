#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::cmp::{Ordering, Reverse};
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

type FullArmors<'a> = HashMap<ArmorPart, CalcArmor<'a>>;

fn calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    sex_type: SexType,
    dm: &'a DataManager,
) -> String {
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

        all_unique_armors
            .get_mut(part)
            .unwrap()
            .sort_by_key(|armor| armor.point());
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
        let (possible_result, possible_combs) = full_equip.get_possible_combs(
            no_deco_skills.clone(),
            &free_slots,
            &no_deco_skills,
            &dm.deco_combinations,
        );

        if possible_result {
            println!(
                "Unique skills possible: {:?}, {:?}",
                possible_combs,
                vec![helm.id(), torso.id(), arm.id(), waist.id(), feet.id()]
            );

            possible_unique_armors.push(armors);
        }
    }

    possible_unique_armors.sort_by_key(|armors| {
        let mut sum = 0;

        for (_, armor) in armors {
            sum += armor.point();
        }

        Reverse(sum)
    });

    let mut total_require_point = 0;

    for (id, level) in &selected_skills {
        match yes_deco_skills.get(id) {
            Some(_) => {
                let mut decos = decos_possible.get(id).unwrap().clone();
                decos.sort_by_key(|deco| deco.slot_size);

                let min_slot_size = decos[0].slot_size;

                total_require_point += level * min_slot_size;
            }
            None => match no_deco_skills.get(id) {
                Some(_) => {
                    total_require_point += level * 990;
                }
                None => (),
            },
        }
    }

    let mut total_index = 0;
    let mut answers = Vec::new();

    'all_cases: for possible_unique_comb in &possible_unique_armors {
        let possible_unique_vec = possible_unique_comb.values().collect::<Vec<&CalcArmor>>();

        println!(
            "Parts sorted id: {} {} {} {} {}",
            possible_unique_vec[0].id(),
            possible_unique_vec[1].id(),
            possible_unique_vec[2].id(),
            possible_unique_vec[3].id(),
            possible_unique_vec[4].id(),
        );

        let mut parts = possible_unique_vec
            .iter()
            .map(|armor| {
                let part = armor.part();

                let mut ret = Vec::new();

                if armor.id().starts_with(EMPTY_ARMOR_PREFIX) {
                    let part_unique_armors = &mut all_unique_armors[&part].clone();
                    let part_deco_armors = &mut armors_with_deco_skills[&part].clone();
                    let part_slot_armors = &mut all_slot_armors[&part]
                        .iter()
                        .map(|(_, armor)| armor.clone())
                        .collect::<Vec<CalcArmor>>();

                    part_unique_armors.sort_by_key(|armor| armor.point());
                    part_deco_armors.sort_by_key(|armor| armor.point());
                    part_slot_armors.sort_by_key(|armor| armor.point());

                    ret.append(part_unique_armors);
                    ret.append(part_deco_armors);
                    ret.append(part_slot_armors);
                } else {
                    ret.push((*armor).clone());
                }

                ret
            })
            .collect::<Vec<Vec<CalcArmor>>>();

        parts.sort_by_key(|p| Reverse(p[0].point()));

        let mut total_count = 1;

        for part_armors in &parts {
            total_count *= part_armors.len();
        }

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

        let mut failed_slot_armors = HashSet::<String>::new();

        let highest_points = vec![
            parts[0][0].point(),
            parts[1][0].point(),
            parts[2][0].point(),
            parts[3][0].point(),
            parts[4][0].point(),
        ];

        let pre_check_point = |index: i32, point: i32| -> i32 {
            let pre_highest: i32 =
                highest_points.iter().sum::<i32>() + point - highest_points[index as usize];

            pre_highest
        };

        for p0 in &parts[0] {
            if pre_check_point(0, p0.point()) < total_require_point {
                println!(
                    "Part 0 highest: {} {}",
                    total_require_point,
                    pre_check_point(0, p0.point()),
                );
                //continue;
            }

            for p1 in &parts[1] {
                if pre_check_point(1, p1.point()) < total_require_point {
                    println!(
                        "Part 1 highest: {} {}",
                        total_require_point,
                        pre_check_point(1, p1.point()),
                    );
                    //continue;
                }

                for p2 in &parts[2] {
                    if pre_check_point(2, p2.point()) < total_require_point {
                        println!(
                            "Part 2 highest: {} {}",
                            total_require_point,
                            pre_check_point(2, p2.point()),
                        );
                        //continue;
                    }

                    for p3 in &parts[3] {
                        if pre_check_point(3, p3.point()) < total_require_point {
                            println!(
                                "Part 3 highest: {} {}",
                                total_require_point,
                                pre_check_point(3, p3.point()),
                            );
                            //continue;
                        }

                        'final_armor: for p4 in &parts[4] {
                            total_index += 1;

                            for failed_id in failed_slot_armors.clone() {
                                if DecorationCombinations::compare(
                                    &BaseArmor::parse_slot_armor_id(&failed_id),
                                    &p4.slots(),
                                ) != Ordering::Less
                                {
                                    continue;
                                }
                            }

                            let mut armors = FullArmors::<'a>::new();

                            armors.insert(p0.part(), p0.clone());
                            armors.insert(p1.part(), p1.clone());
                            armors.insert(p2.part(), p2.clone());
                            armors.insert(p3.part(), p3.clone());
                            armors.insert(p4.part(), p4.clone());

                            let result = calculate_full_equip(
                                dm,
                                &p4,
                                &free_slots,
                                &selected_skills,
                                &no_deco_skills,
                                &weapon_slots,
                                armors,
                                &mut failed_slot_armors,
                                &mut answers,
                                &mut total_index,
                            );

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

    println!(
        "All combinations size: {}, answers size: {}",
        total_index + 1,
        answers.len()
    );

    let elapsed = start_time.elapsed();

    let ret = format!("calculate_skillset elapsed: {:?}", elapsed);
    println!("{}", ret);

    return ret;
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
    p4: &CalcArmor,
    free_slots: &Vec<i32>,
    req_skills: &HashMap<String, i32>,
    no_deco_skills: &HashMap<String, i32>,
    weapon_slots: &Vec<i32>,
    armors: FullArmors<'a>,
    failed_slot_armors: &mut HashSet<String>,
    answers: &mut Vec<(FullEquipments<'a>, DecorationCombination)>,
    total_index: &mut i32,
) -> i32 {
    for slot_count in free_slots {
        if 0 < *slot_count && BaseArmor::is_slot_armor(p4.id()) {
            compare_failed_slot_armors(failed_slot_armors, &p4);

            return 1;
        }
    }
    let full_equip = FullEquipments::new(weapon_slots.clone(), armors, None);

    let (local_result, local_answers) = full_equip.get_possible_combs(
        req_skills.clone(),
        &free_slots,
        &no_deco_skills,
        &dm.deco_combinations,
    );

    if local_result == false {
        compare_failed_slot_armors(failed_slot_armors, &p4);
        return 1;
    }

    println!("Initial slots: {:?}", full_equip.avail_slots);
    println!("Armor ids: {:?}", full_equip.all_skills);

    for local_answer in &local_answers {
        println!("Local answer: {:?}", local_answer);
    }

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

        let final_equip = FullEquipments::new(weapon_slots.clone(), armors, None);

        answers_equip.push(final_equip);
    }

    for (equip, slot_comb) in iproduct!(answers_equip, local_answers) {
        answers.push((equip, slot_comb));
    }

    println!(
        "Answers length: {}, total_index: {}",
        answers.len(),
        total_index
    );
    println!();

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
