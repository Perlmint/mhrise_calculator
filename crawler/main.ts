import path from "path";
import fs from "fs-extra";

import { parse as armorParse } from "./kiranico_armor.js";
import { parse as invenParse, InvenArmorInfo } from "./inven.js";

import { parse as skillParse } from "./kiranico_skill.js";
import { parse as decoParse } from "./kiranico_deco.js";

import {
    ArmorStatInfo,
    FinalArmorInfo,
    ArmorFinalSkillInfo,
} from "./definition/armor_define.js";
import { FinalSkillInfo } from "./definition/skill_define.js";

async function merge() {
    const kiraFile = path.join("temp_data", "armor.json");
    const invenFile = path.join("temp_data", "inven_data.json");
    const armorOverrideFile = path.join("data", "armor_override.json");

    const invenDatas: InvenArmorInfo[] = JSON.parse(
        fs.readFileSync(invenFile).toString()
    );

    const kiraDatas: FinalArmorInfo[] = JSON.parse(
        fs.readFileSync(kiraFile).toString()
    );

    const overrideArmorDatas = JSON.parse(
        fs.readFileSync(armorOverrideFile).toString()
    ) as {
        id: string;
        stat: ArmorStatInfo;
        skills: { [key: string]: ArmorFinalSkillInfo };
        slots: number[];
    }[];

    const realArmorInfos = [] as {
        rarity: number;
        kiraId: number;
        invenId: number;
    }[];

    invenDatas.forEach((invenData, invenIdx) => {
        kiraDatas.some((kiraData, kiraIdx) => {
            if (
                kiraData.names["ko"].replace("【", "[").replace("】", "]") ===
                invenData.name
            ) {
                kiraData.part = invenData.part;
                kiraData.sexType = invenData.sexType;

                for (const ovData of overrideArmorDatas) {
                    if (ovData.id === kiraData.id) {
                        kiraData.stat = ovData.stat;
                        kiraData.skills = ovData.skills;
                        kiraData.slots = ovData.slots;
                    }
                }

                realArmorInfos.push({
                    rarity: invenData.rarity,
                    kiraId: kiraIdx,
                    invenId: invenIdx,
                });

                return true;
            }

            return false;
        });
    });

    const finalArmorDatas = kiraDatas.filter((armorInfo) => {
        if (armorInfo.part === "" || armorInfo.sexType === "") {
            return false;
        }

        return true;
    });

    fs.ensureDirSync("data");

    const armorFilename = path.join("data", `armor.json`);

    const armorJsonStr = JSON.stringify(finalArmorDatas, null, 4);

    const armorProm = new Promise<void>((resolve, reject) => {
        fs.writeFile(armorFilename, armorJsonStr, (err) => {
            if (err) {
                return reject(err);
            }

            resolve();
        });
    });

    const skillDatas = JSON.parse(
        fs.readFileSync(path.join("temp_data", "skill.json")).toString()
    ) as FinalSkillInfo[];

    const overrideSkillDatas = JSON.parse(
        fs.readFileSync(path.join("data", "skill_override.json")).toString()
    ) as { id: string; maxLevel: number }[];

    for (const overInfo of overrideSkillDatas) {
        for (const skillInfo of skillDatas) {
            if (skillInfo.id === overInfo.id) {
                skillInfo.maxLevel = overInfo.maxLevel;
                break;
            }
        }
    }

    const skillFilename = path.join("data", "skill.json");
    const skillJsonStr = JSON.stringify(skillDatas, null, 4);

    const skillProm = new Promise<void>((resolve, reject) => {
        fs.writeFile(skillFilename, skillJsonStr, (err) => {
            if (err) {
                return reject(err);
            }

            resolve();
        });
    });

    fs.copyFileSync(
        path.join("temp_data", "deco.json"),
        path.join("data", "deco.json")
    );

    return Promise.all([armorProm, skillProm]);
}

async function main() {
    const kiranicoParses = async () => {
        await armorParse();
        await skillParse();
        await decoParse();
    };

    await Promise.all([kiranicoParses(), invenParse()]);
    await merge();

    console.log("All data parsing done!");
}

await main();
