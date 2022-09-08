use std::collections::HashMap;
use std::hash::Hash;

use super::armor::{AnomalyArmor, BaseArmor, Talisman};
use super::deco::Decoration;
use super::skill::{self, Skill};

#[derive(Default)]
pub struct DataManager {
    pub armors: HashMap<String, BaseArmor>,
    pub skills: HashMap<String, Skill>,
    pub decos: HashMap<String, Decoration>,

    pub anomaly_armors: Vec<AnomalyArmor>,

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

        for pair in &skills {
            let skill = pair.1;

            for lang_name in &skill.names {
                let name = lang_name.1;

                skill_name_dict.insert(name.to_string(), skill.id.to_string());
            }
        }

        let mut dm = DataManager {
            armors,
            skills,
            decos,
            armor_name_dict,
            skill_name_dict,
            ..Default::default()
        };

        dm
    }
}
