use std::collections::HashMap;

use itertools::izip;

use crate::data::{deco::Decoration, skill::Skill};

#[derive(Default)]
pub struct DecorationCombinations {
    pub combinations: HashMap<String, Vec<Vec<Vec<i32>>>>,
}

impl DecorationCombinations {
    pub fn new(
        decos_by_skill: &HashMap<String, Vec<Decoration>>,
        skills: &HashMap<String, Skill>,
    ) -> DecorationCombinations {
        let mut combinations = HashMap::<String, Vec<Vec<Vec<i32>>>>::new();

        for (id, decos) in decos_by_skill {
            let skill = skills.get(id).unwrap();
            let max_level = skill.max_level;

            if decos.len() == 1 {
                let mut skill_combs = Vec::new();

                let deco_skill_level = decos[0].skill_level;

                for req_level in 1..max_level + 1 {
                    let mut minimum_deco_count = req_level / deco_skill_level + 1;

                    if req_level % deco_skill_level == 0 {
                        minimum_deco_count -= 1;
                    }

                    skill_combs.push(vec![vec![minimum_deco_count]]);
                }

                combinations.insert(id.clone(), skill_combs);
            } else {
                let mut max_deco_counts = Vec::new();
                let mut init_case = Vec::new();

                for deco in decos {
                    let max_required = max_level / deco.skill_level;

                    max_deco_counts.push(max_required);
                    init_case.push(0);
                }

                for req_level in 1..max_level + 1 {
                    let mut skill_temp_combs = Vec::<Vec<i32>>::new();
                    let mut skill_done_combs = Vec::new();

                    skill_temp_combs.push(init_case.clone());

                    for (slot_size_index, max_deco_count) in max_deco_counts.iter().enumerate() {
                        let deco = &decos[slot_size_index];

                        let deco_temp_combs = skill_temp_combs.clone();

                        for temp_comb in &deco_temp_combs {
                            for count in (1..max_deco_count + 1).rev() {
                                let mut cur_level_sum: i32 = temp_comb.iter().sum();
                                cur_level_sum += count * deco.skill_level;

                                let mut next_temp_comb = temp_comb.clone();
                                next_temp_comb[slot_size_index] = count;

                                if req_level <= cur_level_sum {
                                    let mut has_better_slot_answer = false;

                                    for lower_deco_size in 0..slot_size_index {
                                        let lower_deco = &decos[lower_deco_size];

                                        let mut lower_level_sum: i32 = temp_comb.iter().sum();
                                        lower_level_sum += count * lower_deco.skill_level;

                                        if req_level <= lower_level_sum {
                                            has_better_slot_answer = true;
                                            break;
                                        }
                                    }

                                    if has_better_slot_answer == false {
                                        skill_done_combs.push(next_temp_comb);
                                    }
                                } else {
                                    skill_temp_combs.push(next_temp_comb);
                                }
                            }
                        }
                    }

                    combinations.insert(id.clone(), vec![skill_done_combs]);
                }
            }
        }

        for (_, combs) in combinations.iter_mut() {
            for deco_size_combs in combs {
                let mut remove_comb_indices = Vec::new();

                'remove_loop: for index1 in 0..deco_size_combs.len() - 1 {
                    let deco_comb1 = &deco_size_combs[index1];

                    for index2 in index1 + 1..deco_size_combs.len() {
                        let deco_comb2 = &deco_size_combs[index2];

                        let mut is_inferior = true;

                        for (count1, count2) in izip!(deco_comb1, deco_comb2) {
                            if count1 < count2 {
                                is_inferior = false;
                                break;
                            }
                        }

                        if is_inferior {
                            remove_comb_indices.push(index1);
                            continue 'remove_loop;
                        }
                    }
                }

                for remove_index in remove_comb_indices.iter().rev() {
                    deco_size_combs.remove(*remove_index);
                }
            }
        }

        DecorationCombinations { combinations }
    }
}
