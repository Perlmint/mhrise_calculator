use std::collections::HashMap;

use super::armor::{AnomalyArmor, ArmorPart, BaseArmor, Talisman};
use super::deco::Decoration;
use super::skill::Skill;

#[derive(Default)]
pub struct DataManager {
    pub armors: HashMap<String, BaseArmor>,
    pub skills: HashMap<String, Skill>,
    pub decos: HashMap<String, Decoration>,

    pub decos_by_skill: HashMap<String, Vec<Decoration>>,

    pub anomaly_armors: Vec<AnomalyArmor>,

    pub bases_by_part: HashMap<ArmorPart, Vec<BaseArmor>>,
    pub anomalies_by_part: HashMap<ArmorPart, Vec<BaseArmor>>,

    pub talismans: Vec<Talisman>,

    pub armor_name_dict: HashMap<String, String>,
    pub skill_name_dict: HashMap<String, String>,
}

impl DataManager {
    pub fn new(
        armors: HashMap<String, BaseArmor>,
        skills: HashMap<String, Skill>,
        decos: HashMap<String, Decoration>,
    ) -> Self {
        let mut armor_name_dict = HashMap::<String, String>::new();
        let mut skill_name_dict = HashMap::<String, String>::new();

        for pair in &armors {
            let armor = pair.1;

            for lang_name in &armor.names {
                let name = lang_name.1;

                armor_name_dict.insert(name.to_string(), armor.id.to_string());
            }
        }

        for (_, skill) in &skills {
            for lang_name in &skill.names {
                let name = lang_name.1;

                skill_name_dict.insert(name.to_string(), skill.id.to_string());
            }
        }

        let mut decos_by_skill = HashMap::<String, Vec<Decoration>>::new();

        for pair in &decos {
            let deco = pair.1;
            let skill_id = &deco.skill_id;

            let existing = decos_by_skill.get_mut(skill_id);

            match existing {
                Some(vec) => vec.push(deco.clone()),
                None => {
                    decos_by_skill.insert(skill_id.clone(), vec![deco.clone()]);
                }
            }
        }

        for pair in decos_by_skill.iter_mut() {
            pair.1.sort_by_key(|a| a.skill_level);
        }

        let mut bases_by_part = HashMap::<ArmorPart, Vec<BaseArmor>>::new();

        bases_by_part.insert(ArmorPart::Helm, Vec::new());
        bases_by_part.insert(ArmorPart::Torso, Vec::new());
        bases_by_part.insert(ArmorPart::Arm, Vec::new());
        bases_by_part.insert(ArmorPart::Waist, Vec::new());
        bases_by_part.insert(ArmorPart::Feet, Vec::new());

        let anomalies_by_part = bases_by_part.clone();

        for (_, armor) in &armors {
            let part = &armor.part;
            bases_by_part.get_mut(part).unwrap().push(armor.clone());
        }

        let dm = DataManager {
            armors,
            skills,
            decos,
            decos_by_skill,
            armor_name_dict,
            skill_name_dict,
            bases_by_part,
            anomalies_by_part,
            ..Default::default()
        };

        dm
    }

    pub fn set_anomalies(&mut self, anomalies: Vec<AnomalyArmor>) {
        self.anomaly_armors = anomalies;

        for part_armors in self.anomalies_by_part.iter_mut() {
            part_armors.1.clear();
        }

        for anomaly in &self.anomaly_armors {
            let part = &anomaly.original.part;

            self.anomalies_by_part
                .get_mut(part)
                .unwrap()
                .push(anomaly.affected.clone());
        }
    }

    pub fn get_parts(&self, part: ArmorPart) -> Vec<&BaseArmor> {
        let mut ret = Vec::new();

        for part_armors in self.bases_by_part.get(&part).unwrap() {
            ret.push(part_armors);
        }

        for part_anomaly in self.anomalies_by_part.get(&part).unwrap() {
            ret.push(part_anomaly);
        }

        ret
    }

    pub fn get_parts_clone(&self, part: ArmorPart) -> Vec<BaseArmor> {
        let mut ret = Vec::<BaseArmor>::new();

        for part_armors in self.bases_by_part.get(&part).unwrap() {
            ret.push(part_armors.clone());
        }

        for part_anomaly in self.anomalies_by_part.get(&part).unwrap() {
            ret.push(part_anomaly.clone());
        }

        ret
    }

    pub fn get_deco_by_skill_id(&self, skill_id: &String) -> Vec<&Decoration> {
        let existing = self.decos_by_skill.get(skill_id);

        match existing {
            Some(vec) => {
                let mut ret = Vec::<&Decoration>::new();

                for deco in vec {
                    ret.push(deco);
                }

                ret
            }
            None => Vec::new(),
        }
    }

    pub fn has_decoration(&self, skill_id: &String) -> bool {
        self.decos_by_skill.contains_key(skill_id)
    }
}
