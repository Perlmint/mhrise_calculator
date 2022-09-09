<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { FinalSkillInfo } from "../definition/skill_define";

const props = defineProps<{
  index: Number,
  skillsVec: FinalSkillInfo[],
  lang_data: String
}>();

</script>

<template>
  <td>
    <select :name="`anomaly${index}`">
      <option value="" disabled>---</option>
      <option v-for="skillInfo in skillsVec" v-bind:value="skillInfo.id">
        {{ skillInfo.names[lang_data] }}
      </option>
    </select>
  </td>
</template>
