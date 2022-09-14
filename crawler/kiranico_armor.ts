import fs from "fs-extra";
import path from "path";

import Crawler from "crawler";
import { makeId } from "./util.js";

import {
    KiranicoArmorInfo,
    SkillInfo,
    ArmorStatInfo,
    FinalArmorInfo,
    FinalSkillInfo,
    ArmorSlotCount,
} from "./definition/armor_define.js";

interface UrlInfo {
    lang: string;
    rarity: number;
    url: string;
}

type AllInfo = { [key: string]: { [key: number]: KiranicoArmorInfo[] } };

export const langs = [
    "ja",
    "zh",
    "zh-Hant",
    "en",
    "ko",
    "ru",
    "ar",
    "de",
    "es",
    "fr",
    "it",
    "pl",
];

const maxArmorRarity = 10;

const skillParseRegex = new RegExp(/(\d)+$/);

function getRes($: cheerio.CheerioAPI, elem: cheerio.Element) {
    const span = $(elem).children("span").children("span");

    return Number.parseInt(span.text());
}

const allInfos: AllInfo = {};

function crawlCallback(
    lang: string,
    rarity: number,
    err: Error,
    res: Crawler.CrawlerRequestResponse,
    done: () => void
) {
    if (err) {
        console.error(err);
        return done();
    }

    const armorInfos = [] as KiranicoArmorInfo[];

    const $ = res.$;
    const armorRows = $("table tbody tr");

    armorRows.each((i, row) => {
        const cols = $(row).children("td");

        const nameCol = cols[2];
        const slotCol = cols[3];
        const defCol1 = cols[4];
        const defCol2 = cols[5];
        const skillCol = cols[6];

        const armorName = $(nameCol).children("a").text();

        const slots = [] as number[];

        $(slotCol)
            .children("img")
            .each((i, img) => {
                const imgSrc = $(img).attr("src");
                const levelStr = imgSrc?.substring(
                    imgSrc.length - 4,
                    imgSrc.length - 5
                );

                const level = Number.parseInt(levelStr!);

                slots.push(level);
            });

        slots.reverse();

        const defVals1 = $(defCol1).children("div");
        const defVals2 = $(defCol2).children("div");

        const defense = Number.parseInt($(defVals1[0]).text());
        const fireRes = getRes($, defVals1[1]);
        const waterRes = getRes($, defVals1[2]);
        const iceRes = getRes($, defVals2[0]);
        const elecRes = getRes($, defVals2[1]);
        const dragonRes = getRes($, defVals2[2]);

        const skills = [] as SkillInfo[];

        $(skillCol)
            .children("div")
            .each((i, div) => {
                const skillName = $(div).children("a").text();
                const levelFullText = $(div).text();
                const levelText = levelFullText.match(skillParseRegex)?.[0];
                const level = Number.parseInt(levelText!);

                const info = { name: skillName, level } as SkillInfo;

                skills.push(info);
            });

        const statInfo = {
            defense,
            fireRes,
            waterRes,
            iceRes,
            elecRes,
            dragonRes,
        } as ArmorStatInfo;

        const info = {
            name: armorName,
            rarity,
            stat: statInfo,
            skills,
            slots,
        } as KiranicoArmorInfo;

        armorInfos.push(info);
    });

    allInfos[lang][rarity] = armorInfos;

    console.log(`Kiranico parsing (lang: ${lang}, rarity: ${rarity}) done`);

    return done();
}

export async function parse() {
    console.log("Kiranico parsing begin");

    const c = new Crawler({ rateLimit: 1000 });

    const urlInfos = [] as UrlInfo[];

    for (const lang of langs) {
        allInfos[lang] = {};
    }

    for (const lang of langs) {
        for (let rarity = 0; rarity < maxArmorRarity; ++rarity) {
            const url = `https://mhrise.kiranico.com/${lang}/data/armors?view=${rarity}`;
            const info = {
                lang,
                rarity,
                url,
            } as UrlInfo;

            urlInfos.push(info);
        }
    }

    for (const info of urlInfos) {
        c.queue({
            uri: info.url,
            callback: (err, res, done) => {
                crawlCallback(info.lang, info.rarity, err, res, done);
            },
        });
    }

    const baseDir = path.join("temp_data");

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            fs.ensureDirSync(baseDir);

            const finalInfos = [] as FinalArmorInfo[];

            const enData = allInfos["en"];

            for (let i = 0; i < maxArmorRarity; ++i) {
                const enRarityArmors = enData[i];

                enRarityArmors.forEach((enArmorInfo, armorIndex) => {
                    const names = {} as { [key: string]: string };

                    for (const lang of langs) {
                        const langName =
                            allInfos[lang][enArmorInfo.rarity][armorIndex].name;
                        names[lang] = langName;
                    }

                    const skills = {} as { [key: string]: FinalSkillInfo };

                    enArmorInfo.skills.forEach((info) => {
                        const id = makeId(info.name);

                        skills[id] = { level: info.level };
                    });

                    const slots = [] as number[];

                    for (let i = 0; i < ArmorSlotCount; ++i) {
                        slots.push(0);
                    }

                    for (const i in enArmorInfo.slots) {
                        slots[i] = enArmorInfo.slots[i];
                    }

                    const finalInfo = {
                        id: makeId(enArmorInfo.name),
                        part: "",
                        sexType: "",
                        names,
                        rarity: enArmorInfo.rarity,
                        stat: enArmorInfo.stat,
                        skills,
                        slots,
                    } as FinalArmorInfo;

                    finalInfos.push(finalInfo);
                });
            }

            fs.writeFile(
                path.join(baseDir, "armor.json"),
                JSON.stringify(finalInfos, null, 4),
                (err) => {
                    if (err) {
                        reject(err);
                    } else {
                        resolve();
                    }
                }
            );
        });
    });
}
