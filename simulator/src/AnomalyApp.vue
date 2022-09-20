<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { ref, Ref } from "vue";
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api/tauri";

import ArmorsVec from "./data/armor.json";
import SkillsVec from "./data/skill.json";

import { FinalArmorInfo, ArmorStatInfo, ArmorParts, ArmorFinalSkillInfo } from "./definition/armor_define";
import { FinalSkillInfo } from "./definition/skill_define";

import NewAnomalyArmor from "./components/NewAnomalyArmor.vue";


interface AnomalyArmorInfo {
    original: FinalArmorInfo,
    statDiff: ArmorStatInfo,
    slotDiffs: number[],
    skillDiffs: {[key: string]: ArmorFinalSkillInfo},
}

interface TalismanInfo {
  skills: {id: string, level: number}[],
  slot_sizes: number[]
}

let lang_data = ref("ko");

let skills = ref({}) as Ref<{[key: string]: FinalSkillInfo}>;

let skillsVec = ref(SkillsVec as FinalSkillInfo[]) as Ref<FinalSkillInfo[]>;
let armorsVec = ref([]) as Ref<FinalArmorInfo[]>;

for(const armor of ArmorsVec) {
  if(7 <= armor.rarity) {
    armorsVec.value.push(armor);
  }
}

skillsVec.value.sort((elem1, elem2) => elem1.names[lang_data.value] > elem2.names[lang_data.value] ? 1 : -1);
armorsVec.value.sort((elem1, elem2) => elem1.names[lang_data.value] > elem2.names[lang_data.value] ? 1 : -1);

for(const skill of skillsVec.value) {
  skills.value[skill.id] = skill;
}

const parts = ref(ArmorParts);

let armorsByPart = ref({} as {[key: string]: {[key: string]: FinalArmorInfo}});
let armorsByPartVec = ref({} as {[key: string]: FinalArmorInfo[]});

for (const part of parts.value) {
  armorsByPart.value[part] = {};
  armorsByPartVec.value[part] = [];
}

for(const armor of armorsVec.value) {
  const id = armor.id;
  const part = armor.part;

  armorsByPart.value[part][id] = armor;
  armorsByPartVec.value[part].push(armor);
}

for(const part in armorsByPartVec.value) {
  const subArmors = armorsByPartVec.value[part];
  subArmors.sort((elem1, elem2) => elem1.names[lang_data.value] > elem2.names[lang_data.value] ? 1 : -1);
}

let anomaly_filename = ref("");
let anomalyArmors = ref([]) as Ref<AnomalyArmorInfo[]>;
let anomalyArmorsByPart = ref({}) as Ref<{[key: string]: AnomalyArmorInfo[]}>;
let max_anomaly_skills = ref(5);

let talisman_filename = ref("");
let talismans = ref([]) as Ref<TalismanInfo[]>;
let max_talisman_skills = ref(2);

let selectedArmorId = ref("");

async function get_anomaly_file() {
  const file = await open({
    multiple: false,
    directory: false,
    filters: [{
      name: "anomaly_crafting_list",
      extensions: ["txt"]
    }]
  });

  if(file !== null && !Array.isArray(file)) {
    anomaly_filename.value = file;
    
    parse_anomaly_file(file);
  }
}

async function get_talisman_file() {
  const file = await open({
    multiple: false,
    directory: false,
    filters: [{
      name: "talisman_list",
      extensions: ["txt"]
    }]
  });

  if(file !== null && !Array.isArray(file)) {
    anomaly_filename.value = file;
    
    parse_talisman_file(file);
  }
}

async function parse_anomaly_file(filename: string) {
  console.log(`Anomaly filename: ${filename}`);

  anomalyArmors.value = await invoke("cmd_parse_anomaly", { filename });
  anomalyArmors.value.sort((armor1, armor2) => armor1.original.names[lang_data.value] > armor2.original.names[lang_data.value] ? 1 : -1);

  for(const armor of anomalyArmors.value) {
    const part = armor.original.part;

    if (anomalyArmorsByPart.value[part] === undefined) {
      anomalyArmorsByPart.value[part] = [];
    }

    anomalyArmorsByPart.value[part].push(armor);
  }

  console.log(anomalyArmorsByPart.value);
  
  for(const armor of anomalyArmors.value) {
    max_anomaly_skills.value = Math.max(max_anomaly_skills.value, Object.keys(armor.skillDiffs).length);
  }
}

async function parse_talisman_file(filename: string) {
  console.log(`Talisman filename: ${filename}`);

  talismans.value = await invoke("cmd_parse_talisman", { filename });

  console.log(talismans.value);
}

</script>

<template>
  <div class="container">
    <h1>Welcome to Tauri!</h1>

    <button @click="get_anomaly_file()">Load anomaly file</button>

    <input v-model="anomaly_filename" placeholder="Anomaly crafting filename (exported via mod)" />

    <button @click="parse_anomaly_file(anomaly_filename)">Parse Anomaly</button>

    <button @click="get_talisman_file()">Load talisman file</button>

    <input v-model="anomaly_filename" placeholder="Talisman list filename (exported via mod)" />

    <button @click="parse_talisman_file(talisman_filename)">Parse Talisman</button>

    <template v-for="part in parts">
      <table>
        <tr>
          <th><h1>{{ part }}</h1></th>
          <template v-for="i in max_anomaly_skills">
            <th colspan="2">Skill {{ i }}</th>
          </template>
        </tr>

        <tr v-for="armor in anomalyArmorsByPart[part]">
          <td>{{ armor.original.names[lang_data] }}</td>

          <template v-for="(skillInfo, skillId) in armor.skillDiffs">
            <td>{{ skills[skillId].names[lang_data] }}</td>
            <td>Lv {{ skillInfo.level }}</td>
          </template>
        </tr>
      </table>
    </template>

    <table>
      <tr>
        <template v-for="i in max_talisman_skills">
          <th colspan="2">Skill {{ i }}</th>
        </template>
      </tr>

      <tr v-for="tali in talismans">
        <template v-for="skillInfo in tali.skills">
          <td>{{ skills[skillInfo.id].names[lang_data] }}</td>
          <td>Lv {{ skillInfo.level }}</td>
        </template>
      </tr>
    </table>

    <table>
      <tr>
        <th>Name</th>
        <template v-for="i in max_anomaly_skills">
          <th colspan="2">Skill {{ i }}</th>
        </template>
      </tr>
      <tr>
        <td>
          <select :name="`armor_select`" v-model="selectedArmorId">
          <option value="" disabled>---</option>
          <option v-for="armorInfo in armorsVec" :value="armorInfo.id">
            {{ armorInfo.names[lang_data] }}
          </option>
        </select>
        </td>
        <NewAnomalyArmor :index="0" :skillsVec="skillsVec" :skills="skills" :lang_data="lang_data" />
        <NewAnomalyArmor :index="1" :skillsVec="skillsVec" :skills="skills" :lang_data="lang_data" />
        <NewAnomalyArmor :index="2" :skillsVec="skillsVec" :skills="skills" :lang_data="lang_data" />
        <NewAnomalyArmor :index="3" :skillsVec="skillsVec" :skills="skills" :lang_data="lang_data" />
        <NewAnomalyArmor :index="4" :skillsVec="skillsVec" :skills="skills" :lang_data="lang_data" />
      </tr>
    </table>


  </div>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
