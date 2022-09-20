<script setup lang="ts">
import { ref, Ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import SkillCategories from "./data/skill_category.json";
import SkillsVec from "./data/skill.json";

import { SkillCategory } from "./definition/skill_category_define";
import { FinalSkillInfo } from "./definition/skill_define";

const lang_data = ref("ko");

const skillCats = ref(SkillCategories) as Ref<SkillCategory[]>;
const skillsVec = ref(SkillsVec) as Ref<FinalSkillInfo[]>;

const skills = ref({}) as Ref<{[key: string]: FinalSkillInfo}>;

const weaponSlots = ref([0,0,0]) as Ref<number[]>;
const allSkillSelections = ref({}) as Ref<{[key: string]: number}>;
const freeSlots = ref([0,0,0,0]) as Ref<number[]>;

const prevCalcInputStr = window.localStorage.getItem("calc_input");

if (prevCalcInputStr) {
  const prevCalcInput = JSON.parse(prevCalcInputStr);
  weaponSlots.value = prevCalcInput.weaponSlots;
  freeSlots.value = prevCalcInput.freeSlots;
  const prevSelectedSkills = prevCalcInput.selectedSkills as {[key:string]:number};

  for(const skillId in prevSelectedSkills) {
    let level = prevSelectedSkills[skillId];
    
    if(level !== 0) {
      allSkillSelections.value[skillId] = level;
    }
  }
}


const calc_answers = ref("");

for(const skill of skillsVec.value) {
  skills.value[skill.id] = skill;
  allSkillSelections.value[skill.id] = 0;
}

for(const cat of skillCats.value) {
  cat.skills.sort((id1, id2) => skills.value[id1].names[lang_data.value] > skills.value[id2].names[lang_data.value] ? 1 : -1);
}

async function calculate()
{
  const selectedSkills = {} as {[key:string]: number};

  for(const skillId in allSkillSelections.value) {
    let level = allSkillSelections.value[skillId];
    
    if(level !== 0) {
      selectedSkills[skillId] = level;
    }
  }
  
  const calcInput = {
    weaponSlots: weaponSlots.value,
    selectedSkills,
    freeSlots: freeSlots.value
  };

  window.localStorage.setItem("calc_choices", JSON.stringify(calcInput));

  console.log(calcInput);

  const result = await invoke("cmd_calculate_skillset", calcInput) as {[key:string]:any};

  calc_answers.value = result["log"] as string;

  console.log(result);
}

</script>

<template>
  <table>
    <tr>
      <td>
        무기 슬롯
      </td>
      <td>
        Slot 1
        <input type="radio" v-model="weaponSlots[0]" :value="0">
        <input type="radio" v-for="level in 4" v-model="weaponSlots[0]" :value="level">
      </td>
      <td>
        Slot 2
        <input type="radio" v-model="weaponSlots[1]" :value="0">
        <input type="radio" v-for="level in 4" v-model="weaponSlots[1]" :value="level">
      </td>
      <td>
        Slot 3
        <input type="radio" v-model="weaponSlots[2]" :value="0">
        <input type="radio" v-for="level in 4" v-model="weaponSlots[2]" :value="level">
      </td>
    </tr>
  </table>

  <br />

  <table v-for="cat in skillCats">
    <tr>
      {{ cat.names[lang_data] }}
    </tr>
    <tr>
      <div>
        <span v-for="id in cat.skills">
          {{ skills[id].names[lang_data] }}
          <select :name="id" v-model="allSkillSelections[id]" >
            <option :value="0" selected>---</option>
            <option v-for="level in skills[id].maxLevel" :value="level">
              Lv {{ level }}
            </option>
          </select>
        </span>
      </div>
      <br />
    </tr>
  </table>

  <table>
    <tr>
      <td>Free slots count</td>
      <td>
        <select name="slots_lv1" v-model.number="freeSlots[0]">
          <option :value="0">0</option>
          <option v-for="count in 10" :value="count">
            {{ count }}
          </option>
        </select>
      </td>
      <td>
        <select name="slots_lv2" v-model.number="freeSlots[1]">
          <option :value="0">0</option>
          <option v-for="count in 10" :value="count">
            {{ count }}
          </option>
        </select>
      </td>
      <td>
        <select name="slots_lv3" v-model.number="freeSlots[2]">
          <option :value="0">0</option>
          <option v-for="count in 10" :value="count">
            {{ count }}
          </option>
        </select>
      </td>
      <td>
        <select name="slots_lv4" v-model="freeSlots[3]">
          <option :value="0">0</option>
          <option v-for="count in 10" :value="count">
            {{ count }}
          </option>
        </select>
      </td>
    </tr>
  </table>

  <button @click="calculate">Calculate</button>

  <textarea v-model="calc_answers"></textarea>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
