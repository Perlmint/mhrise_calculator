import path from "path";
import fs from "fs-extra";

import {
    parse as armorParse,
    langs,
    KiranicoArmorInfo,
} from "./kiranico_armor.js";
import { parse as invenParse, InvenArmorInfo } from "./inven.js";

import { parse as skillParse } from "./kiranico_skill.js";
import { parse as decoParse } from "./kiranico_deco.js";

interface FinalArmorInfo extends KiranicoArmorInfo {
    id: number;
    part: string;
    sexType: string;
}

async function merge() {
    const kiranicoFiles: { [key: string]: string } = {};

    langs.forEach((lang) => {
        kiranicoFiles[lang] = path.join(
            "temp_data",
            "armor",
            `armor.${lang}.json`
        );
    });

    const invenFile = path.join("temp_data", "inven_data.json");

    const invenDatas: InvenArmorInfo[] = JSON.parse(
        fs.readFileSync(invenFile).toString()
    );
    const kiraKoDatas: { [key: string]: FinalArmorInfo[] } = JSON.parse(
        fs.readFileSync(kiranicoFiles["ko"]).toString()
    );

    const realArmorInfos = [] as {
        rarity: number;
        kiraId: number;
        invenId: number;
    }[];

    invenDatas.forEach((invenData, invenIdx) => {
        const rarityArmors = kiraKoDatas[invenData.rarity - 1];

        for (let kiraIdx = 0; kiraIdx < rarityArmors.length; ++kiraIdx) {
            const info = rarityArmors[kiraIdx];

            if (info.name === invenData.name) {
                info.id = invenIdx;
                info.part = invenData.part;
                info.sexType = invenData.sexType;

                realArmorInfos.push({
                    rarity: invenData.rarity,
                    kiraId: kiraIdx,
                    invenId: invenIdx,
                });

                break;
            }
        }
    });

    const allLangDatas: { [key: string]: { [key: string]: FinalArmorInfo[] } } =
        {};

    allLangDatas["ko"] = kiraKoDatas;

    for (const lang in kiranicoFiles) {
        if (lang === "ko") {
            continue;
        }

        const otherLangDatas: { [key: string]: FinalArmorInfo[] } = JSON.parse(
            fs.readFileSync(kiranicoFiles[lang]).toString()
        );

        allLangDatas[lang] = otherLangDatas;

        realArmorInfos.forEach((info) => {
            const otherArmor = otherLangDatas[info.rarity - 1][info.kiraId];
            const koArmor = kiraKoDatas[info.rarity - 1][info.kiraId];

            otherArmor.id = koArmor.id;
            otherArmor.part = koArmor.part;
            otherArmor.sexType = koArmor.sexType;
        });
    }

    fs.ensureDirSync("data");

    const proms = [] as Promise<void>[];

    for (const lang of langs) {
        const filename = path.join("data", `data.${lang}.json`);

        const datas = [] as FinalArmorInfo[];
        const langDatas = allLangDatas[lang];

        realArmorInfos.forEach((info) => {
            datas.push(langDatas[info.rarity - 1][info.kiraId]);
        });

        const jsonStr = JSON.stringify(datas, null, 4);

        const prom = new Promise<void>((resolve, reject) => {
            fs.writeFile(filename, jsonStr, (err) => {
                if (err) {
                    return reject(err);
                }

                resolve();
            });
        });

        proms.push(prom);
    }

    await Promise.all(proms);
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
