import path from "path";
import fs from "fs-extra";

import { parse as armorParse, FinalArmorInfo } from "./kiranico_armor.js";
import { parse as invenParse, InvenArmorInfo } from "./inven.js";

import { parse as skillParse } from "./kiranico_skill.js";
import { parse as decoParse } from "./kiranico_deco.js";

async function merge() {
    const kiraFile = path.join("temp_data", "armor.json");
    const invenFile = path.join("temp_data", "inven_data.json");

    const invenDatas: InvenArmorInfo[] = JSON.parse(
        fs.readFileSync(invenFile).toString()
    );
    const kiraDatas: FinalArmorInfo[] = JSON.parse(
        fs.readFileSync(kiraFile).toString()
    );

    const realArmorInfos = [] as {
        rarity: number;
        kiraId: number;
        invenId: number;
    }[];

    invenDatas.forEach((invenData, invenIdx) => {
        kiraDatas.some((kiraData, kiraIdx) => {
            if (kiraData.names["ko"] === invenData.name) {
                kiraData.part = invenData.part;
                kiraData.sexType = invenData.sexType;

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

    fs.ensureDirSync("data");

    const filename = path.join("data", `armor.json`);

    const jsonStr = JSON.stringify(kiraDatas, null, 4);

    const prom = new Promise<void>((resolve, reject) => {
        fs.writeFile(filename, jsonStr, (err) => {
            if (err) {
                return reject(err);
            }

            resolve();
        });
    });

    fs.copyFileSync(
        path.join("temp_data", "skill.json"),
        path.join("data", "skill.json")
    );
    fs.copyFileSync(
        path.join("temp_data", "deco.json"),
        path.join("data", "deco.json")
    );

    return prom;
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
