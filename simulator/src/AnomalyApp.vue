<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { onBeforeMount, ref } from "vue";
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api/tauri";

import ArmorsVec from "./data/armor.json";
import SkillsVec from "./data/skill.json";

import { FinalArmorInfo, ArmorStatInfo } from "./definition/armor_define";
import { FinalSkillInfo } from "./definition/skill_define";

import NewAnomalyArmor from "./components/NewAnomalyArmor.vue";


interface AnomalyArmorInfo {
    original: FinalArmorInfo,
    statDiff: ArmorStatInfo,
    slotDiffs: number[],
    skillDiffs: FinalSkillInfo[],
}

let lang_data = ref("ko");

let skills = ref({} as {[key: string]: FinalSkillInfo});

let skillsVec = ref(SkillsVec as FinalSkillInfo[]);
let armorsVec = ref(ArmorsVec as FinalArmorInfo[]);

skillsVec.value.sort((elem1, elem2) => elem1.names[lang_data.value] > elem2.names[lang_data.value] ? 1 : -1);
armorsVec.value.sort((elem1, elem2) => elem1.names[lang_data.value] > elem2.names[lang_data.value] ? 1 : -1);

for(const skill of SkillsVec) {
  skills.value[skill.id] = skill;
}

let anomaly_filename = ref("");
let anomaly_armors = ref([] as AnomalyArmorInfo[]);
let max_anomaly_skills = ref(5);

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

async function parse_anomaly_file(filename: string) {
  console.log(`Anomaly filename: ${filename}`);

  anomaly_armors.value = await invoke("cmd_parse_anomaly", { filename });

  anomaly_armors.value.sort((armor1, armor2) => armor1.original.names[lang_data.value] > armor2.original.names[lang_data.value] ? 1 : -1);
  
  for(const armor of anomaly_armors.value) {
    max_anomaly_skills.value = Math.max(max_anomaly_skills.value, armor.skillDiffs.length);
  }
}

function onArmorChange(event: Event) {
  console.log(event.target);
}

</script>

<template>
  <div class="container">
    <h1>Welcome to Tauri!</h1>

    <button @click="get_anomaly_file()">Load anomaly file</button>

    <input v-model="anomaly_filename" placeholder="Anomaly crafting filename (exported via mod)" />

    <button @click="parse_anomaly_file(anomaly_filename)">Parse</button>

    <table>
      <tr>
        <th>Name</th>
        <template v-for="i in max_anomaly_skills">
          <th colspan="2">Skill {{ i }}</th>
        </template>
      </tr>
      <tr v-for="(armor, armorIdx) in anomaly_armors">
        <td>{{ armor.original.names[lang_data] }}</td>

        <template v-for="(skillDiff, skillIdx) in armor.skillDiffs">
          <td>{{ skills[skillDiff.id].names[lang_data] }}</td>
          <td>Lv {{ skillDiff.level }}</td>
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
        <NewAnomalyArmor :index="0" :skillsVec="skillsVec" :lang_data="lang_data" />
        <td>Level1 dropdown</td>
        <NewAnomalyArmor :index="1" :skillsVec="skillsVec" :lang_data="lang_data" />
        <td>Level2 dropdown</td>
        <NewAnomalyArmor :index="2" :skillsVec="skillsVec" :lang_data="lang_data" />
        <td>Level3 dropdown</td>
        <NewAnomalyArmor :index="3" :skillsVec="skillsVec" :lang_data="lang_data" />
        <td>Level4 dropdown</td>
        <NewAnomalyArmor :index="4" :skillsVec="skillsVec" :lang_data="lang_data" />
        <td>Level5 dropdown</td>
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
