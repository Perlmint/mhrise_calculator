import path from "path";
import fs from "fs";

import {
    parse as kiranicoParse,
    langs,
    KiranicoArmorInfo,
} from "./kiranico.js";
import { parse as invenParse, InvenArmorInfo } from "./inven.js";

interface FinalArmorInfo extends KiranicoArmorInfo {
    id: number;
    part: string;
}

function merge() {
    const kiranicoFiles: { [key: string]: string } = {};

    langs.forEach((lang) => {
        kiranicoFiles[lang] = path.join("data", `kira_data.${lang}.json`);
    });

    const invenFile = path.join("data", "inven_data.json");

    const invenDatas: InvenArmorInfo[] = JSON.parse(
        fs.readFileSync(invenFile).toString()
    );
    const kiraKoDatas: { [key: string]: FinalArmorInfo[] } = JSON.parse(
        fs.readFileSync(kiranicoFiles["ko"]).toString()
    );

    invenDatas.forEach((invenData, i) => {
        const rarityArmors = kiraKoDatas[invenData.rarity];

        for (const info of rarityArmors) {
            if (info.name == invenData.name) {
                info.id = i;
                info.part = invenData.part;

                break;
            }
        }
    });

    for (const rarity in kiraKoDatas) {
        const koArmors = kiraKoDatas[rarity];

        for (const lang in kiranicoFiles) {
            if (lang == "ko") {
                continue;
            }

            const otherLangDatas: { [key: string]: FinalArmorInfo[] } =
                JSON.parse(fs.readFileSync(kiranicoFiles[lang]).toString());

            koArmors.forEach((koArmor, i) => {
                for (const rarity in otherLangDatas) {
                }
            });
        }
    }
}

async function main() {
    // await Promise.all([kiranicoParse(), invenParse()]);
    await invenParse();

    merge();
}

await main();
