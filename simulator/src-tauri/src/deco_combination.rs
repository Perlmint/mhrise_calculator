use std::collections::HashMap;

use crate::data::{deco::Decoration, skill::Skill};

pub struct DecorationCombinations {
    pub combinations: HashMap<String, Vec<Vec<i32>>>,
}

impl DecorationCombinations {
    pub fn calculate(
        &mut self,
        decos_by_skill: &HashMap<String, Vec<Decoration>>,
        skills: &HashMap<String, Skill>,
    ) {
        for (id, decos) in decos_by_skill {
            let mut skill_combs = Vec::new();

            let skill = skills.get(id).unwrap();
            let max_level = skill.max_level;

            if decos.len() == 1 {
                let deco_skill_level = decos[0].skill_level;

                for req_level in 1..max_level + 1 {
                    let mut minimum_deco_count = req_level / deco_skill_level + 1;

                    if req_level % deco_skill_level == 0 {
                        minimum_deco_count -= 1;
                    }

                    skill_combs.push(vec![minimum_deco_count]);
                }

                self.combinations.insert(id.clone(), skill_combs);
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

                    self.combinations.insert(id.clone(), skill_done_combs);
                }
            }
        }

        panic!("All deco combs: {:?}", self.combinations);
    }
}
