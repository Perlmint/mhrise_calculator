<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { FinalSkillInfo } from "../definition/skill_define";

const props = defineProps<{
  index: Number,
  skillsVec: FinalSkillInfo[],
  lang_data: String
}>();

console.log(`AnomalyApp skillsVec: ${props.skillsVec.length}`);

const selectedSkillId = ref("");

defineEmits<{
  (event: "onSkillChange", index: Number, id: String): void
}>();

</script>

<template>
  <td>
    <select :name="`anomaly${index}`" v-model="selectedSkillId" @change="$emit('onSkillChange', index, selectedSkillId)">
      <option value="" disabled>---</option>
      <option v-for="skillInfo in skillsVec" v-bind:value="skillInfo.id" :value="skillInfo.id">
        {{ skillInfo.names[lang_data] }}
      </option>
    </select>
  </td>
</template>
