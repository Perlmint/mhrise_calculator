<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { FinalSkillInfo } from "../definition/skill_define";

const props = defineProps<{
  index: number,
  skillsVec: FinalSkillInfo[],
  skills: {[key: string]: FinalSkillInfo},
  lang_data: String
}>();

const selectedSkillId = ref("");
const selectedSkillLevel = ref(0);
const maxLevel = ref(0);

const emit = defineEmits<{
  (event: "onSkillChange", index: number, id: string): void
}>();

function onSkillChange(index: number, id: string) {
  const skillInfo = props.skills[id];

  if (skillInfo === undefined) {
    maxLevel.value = 0;
  }
  else{
    maxLevel.value = skillInfo.maxLevel;
    selectedSkillLevel.value = 1;
  }

  emit('onSkillChange', index, id);
}

</script>

<template>
  <td>
    <select :name="`anomaly${index}`" v-model="selectedSkillId" @change="onSkillChange(index, selectedSkillId)">
      <option value="" selected="selected">---</option>
      <option v-for="skillInfo in skillsVec" v-bind:value="skillInfo.id" :value="skillInfo.id">
        {{ skillInfo.names[lang_data] }}
      </option>
    </select>
  </td>
  <td>
    <select :name="`level${index}`" v-model="selectedSkillLevel">
      <option value="0">---</option>
      <option v-for="level in maxLevel" :value="level"> {{ level }} </option>
    </select>
  </td>
</template>
