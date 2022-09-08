<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { ref } from "vue";
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api/tauri";

interface ArmorStatInfo {
    defense: number;
    fireRes: number;
    waterRes: number;
    iceRes: number;
    elecRes: number;
    dragonRes: number;
}

interface SkillInfo {
    name: string;
    level: number;
}

interface FinalArmorInfo {
    id: string;
    part: string;
    sexType: string;
    names: { [key: string]: string };
    rarity: number;
    stat: ArmorStatInfo;
    skills: SkillInfo[];
    slots: number[];
}

interface AnomalyArmorInfo {
    original: FinalArmorInfo,
    statDiff: ArmorStatInfo,
    slotDiffs: number[],
    skillDiffs: SkillInfo[],
}

let anomaly_filename = ref("");
let anomaly_armors = ref([] as AnomalyArmorInfo[]);

async function get_anomaly_file() {
  const file = await open({
    multiple: false,
    directory: false,
    filters: [{
      name: "anomaly_crafting_list",
      extensions: ["txt", "*"]
    }]
  });

  if(file !== null && !Array.isArray(file)) {
    anomaly_filename.value = file;
    parse_anomaly_file();
  }
}

async function parse_anomaly_file() {
  console.log(`Anomaly filename: ${anomaly_filename.value}`);

  anomaly_armors.value = await invoke("cmd_parse_anomaly", { filename : anomaly_filename.value });

  anomaly_armors.value.forEach(val => {
      console.log(val.statDiff.defense);
  });
}

</script>

<template>
  <div class="container">
    <h1>Welcome to Tauri!</h1>

    <button @click="get_anomaly_file()">Load anomaly file</button>

    <input v-model="anomaly_filename" placeholder="Anomaly crafting filename (exported via mod)" />

    <button @click="parse_anomaly_file()">Parse</button>


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
