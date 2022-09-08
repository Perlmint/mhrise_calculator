use std::collections::HashMap;

use super::armor::{AnomalyArmor, BaseArmor, Talisman};
use super::deco::Decoration;
use super::skill::Skill;

#[derive(Default)]
pub struct DataManager<'a> {
    pub armors: HashMap<String, BaseArmor>,
    pub skills: HashMap<String, Skill>,
    pub decos: HashMap<String, Decoration>,

    pub anomaly_armors: Vec<AnomalyArmor>,

    pub talismans: Vec<Talisman>,

    pub armor_name_dict: HashMap<&'a str, &'a str>,
    pub skill_name_dict: HashMap<&'a str, &'a str>,
}

impl<'a> DataManager<'a> {
    pub fn new(
        armors: HashMap<String, BaseArmor>,
        skills: HashMap<String, Skill>,
        decos: HashMap<String, Decoration>,
    ) -> Self {
        let mut armor_name_dict = HashMap::<&'a str, &'a str>::new();
        let mut skill_name_dict = HashMap::<&'a str, &'a str>::new();

        for pair in armors.iter() {
            let armor = pair.1;

            for lang_name in &armor.names {
                let name = lang_name.1;

                armor_name_dict.insert(name, &armor.id);
            }
        }

        for pair in &skills {
            let skill = pair.1;

            for lang_name in &skill.names {
                let name = lang_name.1;

                skill_name_dict.insert(name, &skill.id);
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
