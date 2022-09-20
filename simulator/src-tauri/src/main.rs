#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub static MAX_ANSWER_LENGTH: i32 = 200;

use csv::StringRecord;
use log::{debug, info};
use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, HashSet};
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
    pub mod deco;
    pub mod skill;
}

mod full_equipments;

mod test;

use crate::calc::armor::CalcArmor;
use crate::calc::deco::CalcDeco;
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

#[derive(Serialize)]
struct CalculateSkillsetReturn {
    log: String,
}

#[tauri::command]
fn cmd_calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    mutex_dm: tauri::State<'a, Mutex<DataManager>>,
) -> CalculateSkillsetReturn {
    debug!("Start calculating...");

    let dm = mutex_dm.lock().unwrap();

    // TODO get sex_type as input
    let (log, answers) = calculate_skillset(
        weapon_slots,
        selected_skills,
        free_slots,
        SexType::Female,
        &dm,
    );

    CalculateSkillsetReturn { log }
}

fn calculate_skillset<'a>(
    weapon_slots: Vec<i32>,
    selected_skills: HashMap<String, i32>,
    free_slots: Vec<i32>,
    sex_type: SexType,
    dm: &'a DataManager,
) -> (String, Vec<(FullEquipments, DecorationCombination)>) {
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

    debug!("Skills with yes deco: {:?}", yes_deco_skills);
    debug!("Skills with no deco: {:?}", no_deco_skills);

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

        armors.sort_by_key(|armor| {
            armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
        });
    }

    let mut all_slot_armors = HashMap::<ArmorPart, HashMap<String, CalcArmor<'a>>>::new();

    for (part, _) in all_armors.iter() {
        let slot_only_armors = dm.slot_only_armors.get(part).unwrap();
        let mut part_slot_armors = HashMap::<String, CalcArmor<'a>>::new();

        for (id, armor) in slot_only_armors {
            let calc_armor = CalcArmor::<'a>::new(armor);

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
            .sort_by_key(|armor| {
                armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
            });
    }

    let mut possible_unique_armors = Vec::new();

    for (helm, torso, arm, waist, feet) in iproduct!(
        all_unique_armors[&ArmorPart::Helm].iter(),
        all_unique_armors[&ArmorPart::Torso].iter(),
        all_unique_armors[&ArmorPart::Arm].iter(),
        all_unique_armors[&ArmorPart::Waist].iter(),
        all_unique_armors[&ArmorPart::Feet].iter()
    ) {
        let armors = vec![
            helm.clone(),
            torso.clone(),
            arm.clone(),
            waist.clone(),
            feet.clone(),
        ];

        let full_equip = FullEquipments::new(weapon_slots.clone(), armors.clone(), None);
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
                vec![helm.id(), torso.id(), arm.id(), waist.id(), feet.id()]
            );

            possible_unique_armors.push(armors);
        }
    }

    possible_unique_armors.sort_by_key(|armors| {
        let mut sum = 0;

        for armor in armors {
            sum += armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills);
        }

        Reverse(sum)
    });

    ret.push_str(&format!(
        "Unique armors calculation: {:?}\n",
        start_time.elapsed()
    ));

    let mut all_parts = Vec::new();

    for possible_unique_vec in &possible_unique_armors {
        debug!(
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
                        .collect::<Vec<CalcArmor>>();

                    ret.append(part_unique_armors);
                    ret.append(part_deco_armors);
                    ret.append(part_slot_armors);
                } else {
                    ret.push((*armor).clone());
                }

                ret.sort_by_key(|armor| {
                    armor.get_point(&decos_possible, &yes_deco_skills, &no_deco_skills)
                });

                ret
            })
            .collect::<Vec<Vec<CalcArmor>>>();

        parts.sort_by_key(|parts| parts.len());

        let mut total_count = 1;

        for part_armors in &parts {
            total_count *= part_armors.len();
        }

        debug!(
            "Parts size: {} {} {} {} {}, total: {}",
            parts[0].len(),
            parts[1].len(),
            parts[2].len(),
            parts[3].len(),
            parts[4].len(),
            total_count,
        );

        all_parts.push(parts);
    }

    ret.push_str(&format!(
        "Empty armors expand: {:?}\n",
        start_time.elapsed()
    ));

    let mut all_loop_tree = std::collections::BTreeMap::new();

    let mut total_case_count = 0;

    'all_parts: for parts in &all_parts {
        'final_armor: for (p0, p1, p2, p3, p4) in
            iproduct!(&parts[0], &parts[1], &parts[2], &parts[3], &parts[4])
        {
            let mut req_skills = selected_skills.clone();
            let mut req_slots = free_slots.clone();

            let mut real_parts = vec![p0.clone(), p1.clone(), p2.clone(), p3.clone(), p4.clone()];
            let real_ids = real_parts
                .iter()
                .map(|part| part.id().clone())
                .collect::<HashSet<String>>();

            let init_equip = FullEquipments::new(weapon_slots.clone(), real_parts.clone(), None);

            for part in real_parts.iter_mut() {
                part.subtract_skills(&mut req_skills);
            }

            let (no_deco_skills, single_deco_skills, multi_deco_skills) =
                dm.get_skils_by_deco(&req_skills);

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

            let mut full_equip =
                FullEquipments::new(weapon_slots.clone(), real_parts.clone(), None);

            let slot_success = full_equip.subtract_slots(&mut req_slots);

            if slot_success == false {
                continue;
            }

            for (id, _, count) in &single_deco_skills {
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
                continue;
            }

            let has_possible_comb = dm
                .deco_combinations
                .has_possible_combs(&req_skills, &full_equip.avail_slots);

            if has_possible_comb == false {
                continue;
            }

            let total_point = CalcDeco::get_point(&full_equip.avail_slots);

            debug!(
                "Possible candidiate: {:?}\nleft skills: {:?}, slots: {:?}, minimum slots: {}, equip slot sum {}, point: {}",
                real_parts.iter().map(|part| part.id()).collect::<Vec<&String>>(), req_skills, full_equip.avail_slots, minimum_slot_sum,  equip_slot_sum, total_point
            );

            let mut existing = all_loop_tree.get_mut(&Reverse(total_point));

            if existing.is_none() {
                all_loop_tree.insert(Reverse(total_point), Vec::new());
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
                break 'all_parts;
            }
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

    'all_cases: for (_, case_vec) in all_loop_tree.iter() {
        for (full_equip, req_skills) in case_vec {
            total_index += 1;

            let result = calculate_full_equip(
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
        total_index + 1,
        answers.len()
    );

    let elapsed_final = start_time.elapsed();

    ret.push_str(&format!(
        "calculate_skillset elapsed: {:?},\nanswers length: {}\n",
        elapsed_final,
        answers.len()
    ));
    info!("{}", ret);

    return (ret, answers);
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
    answers: &mut Vec<(FullEquipments<'a>, DecorationCombination)>,
    total_index: &mut i32,
) -> i32 {
    let mut local_answers = dm.deco_combinations.get_possible_combs(&req_skills);
    local_answers.retain(|comb| comb.is_possible(&full_equip.avail_slots));

    let local_result = local_answers.len() != 0;

    if local_result == false {
        return 1;
    }

    debug!("Initial slots: {:?}", full_equip.avail_slots);
    debug!("Skill ids: {:?}", full_equip.all_skills);

    for local_answer in &local_answers {
        debug!("Local answer: {:?}", local_answer);
    }

    debug!(
        "Possible slot combinations: {:?} {:?}",
        local_answers
            .iter()
            .map(|comb| &comb.combs_per_skill)
            .collect::<Vec<&HashMap<String, Vec<i32>>>>(),
        local_answers
            .iter()
            .map(|comb| &comb.sum)
            .collect::<Vec<&Vec<i32>>>()
    );

    debug!(
        "Armors ids: ({}), ({}), ({}), ({}), ({})",
        full_equip.get_by_part(&ArmorPart::Helm).id(),
        full_equip.get_by_part(&ArmorPart::Torso).id(),
        full_equip.get_by_part(&ArmorPart::Arm).id(),
        full_equip.get_by_part(&ArmorPart::Waist).id(),
        full_equip.get_by_part(&ArmorPart::Feet).id(),
    );

    let mut real_armors = Vec::<Vec<CalcArmor<'a>>>::new();

    for armor in &full_equip.armors {
        if BaseArmor::is_slot_armor(armor.id()) {
            let all_real_armors = dm
                .armors_by_slot
                .get(&armor.part())
                .unwrap()
                .get(armor.id())
                .unwrap()
                .iter()
                .map(|base_armor| CalcArmor::<'a>::new(base_armor))
                .collect::<Vec<CalcArmor<'a>>>();

            real_armors.push(all_real_armors.clone());
        } else {
            real_armors.push(vec![armor.clone()]);
        }
    }

    let mut answers_equip = Vec::new();

    for (a0, a1, a2, a3, a4) in iproduct!(
        &real_armors[0],
        &real_armors[1],
        &real_armors[2],
        &real_armors[3],
        &real_armors[4]
    ) {
        let final_equip = FullEquipments::new(
            weapon_slots.clone(),
            vec![a0.clone(), a1.clone(), a2.clone(), a3.clone(), a4.clone()],
            None,
        );

        answers_equip.push(final_equip);
    }

    for (equip, slot_comb) in iproduct!(answers_equip, local_answers) {
        answers.push((equip, slot_comb));
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
            cmd_get_skill_names,
            cmd_get_armor_names,
            cmd_calculate_skillset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
