import fs from "fs-extra";
import path from "path";

import Crawler from "crawler";

interface UrlInfo {
    lang: string;
    rarity: number;
    url: string;
}

export interface KiranicoArmorInfo {
    name: string;
    rarity: number;
    stat: ArmorStatInfo;
    skills: SkillInfo[];
}

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

        const slots = [];

        $(slotCol)
            .children("img")
            .each((i, img) => {
                const imgSrc = $(img).attr("src");
                const level = imgSrc?.substring(
                    imgSrc.length - 4,
                    imgSrc.length - 5
                );

                slots.push(level);
            });

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

    const baseDir = path.join("temp_data", "armor");

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            fs.ensureDirSync(baseDir);

            const proms = [] as Promise<void>[];

            for (const lang of langs) {
                const prom = new Promise<void>((resolve, reject) => {
                    const resultStr = JSON.stringify(allInfos[lang], null, 4);

                    fs.writeFile(
                        path.join(baseDir, `armor.${lang}.json`),
                        resultStr,
                        (err) => {
                            if (err) {
                                reject(err);
                            } else {
                                console.log(
                                    `Kiranico ${lang} data file write done`
                                );
                                resolve();
                            }
                        }
                    );
                });

                proms.push(prom);
            }

            Promise.all(proms).then(
                () => resolve(),
                () => reject()
            );
        });
    });
}
