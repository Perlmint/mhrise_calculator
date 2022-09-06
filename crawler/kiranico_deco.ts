import fs from "fs-extra";
import path from "path";

import Crawler from "crawler";

import { langs } from "./kiranico_armor.js";
import { makeId } from "./util.js";

interface UrlInfo {
    url: string;
    lang: string;
}

interface DecoInfo {
    name: string;
    skillName: string;
    text: string;
    slotSize: number;
    skillLevel: number;
}

interface FinalDecoInfo {
    id: string;
    names: { [key: string]: string };
    skillNames: { [key: string]: string };
    texts: { [key: string]: string };
    slotSize: number;
    skillLevel: number;
}

const allInfos: { [key: string]: DecoInfo[] } = {};

const slotSizeRegex = new RegExp(/(\d+)/);
const skillLevelRegex = new RegExp(/(\d+)\s*$/);

function crawlCallback(
    lang: string,
    err: Error,
    res: Crawler.CrawlerRequestResponse,
    done: () => void
) {
    if (err) {
        console.error(err);

        return done();
    }

    const $ = res.$;

    const decos = $("table tbody tr");

    const infos = [] as DecoInfo[];

    decos.each((i, elem) => {
        const tds = $(elem).children("td");

        const decoName = $(tds[0]).children("a").text();

        const slotSizeMatch = decoName.normalize("NFKC").match(slotSizeRegex);
        const slotSize = Number.parseInt(slotSizeMatch![1]);

        const skillName = $(tds[1]).children("div").children("a").text();
        const skillLevelText = $(tds[1]).children("div").text();

        const skillLevelMatch = skillLevelText.match(skillLevelRegex);

        const skillLevel = Number.parseInt(skillLevelMatch![1]);

        const skillText = $(tds[2]).text();

        const info = {
            name: decoName,
            slotSize,
            skillName,
            skillLevel,
            text: skillText,
        } as DecoInfo;

        infos.push(info);
    });

    allInfos[lang] = infos;

    console.log(`Kiranico deco parsing (lang: ${lang}) done`);
    done();
}

export async function parse() {
    console.log("Kiranico deco parsing begin");

    const c = new Crawler({ rateLimit: 1000 });

    const urlInfos = langs.map(
        (lang) =>
            ({
                url: `https://mhrise.kiranico.com/${lang}/data/decorations`,
                lang: lang,
            } as UrlInfo)
    );

    urlInfos.forEach((info) => {
        c.queue({
            uri: info.url,
            callback: (err, res, done) => {
                crawlCallback(info.lang, err, res, done);
            },
        });
    });

    const baseDir = path.join("temp_data");

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            console.log();

            fs.ensureDirSync(baseDir);

            const finalInfos = [] as FinalDecoInfo[];

            const enInfos = allInfos["en"];

            enInfos.forEach((enInfo, index) => {
                const names = {} as { [key: string]: string };
                const skillNames = {} as { [key: string]: string };
                const texts = {} as { [key: string]: string };

                langs.forEach((lang) => {
                    const info = allInfos[lang][index];
                    names[lang] = info.name;
                    skillNames[lang] = info.skillName;
                    texts[lang] = info.text;
                });

                const finalInfo = {
                    id: `${makeId(enInfo.name)}_${enInfo.skillLevel}`,
                    names,
                    skillNames,
                    texts,
                    skillLevel: enInfo.skillLevel,
                    slotSize: enInfo.slotSize,
                } as FinalDecoInfo;

                finalInfos.push(finalInfo);
            });

            const filename = path.join(baseDir, "deco.json");
            const dataStr = JSON.stringify(finalInfos, null, 4);

            fs.writeFile(filename, dataStr, (err) => {
                if (err) {
                    return reject(err);
                }

                console.log(`Kiranico deco data file write done`);
                resolve();
            });
        });
    });
}
