import fs from "fs-extra";
import path from "path";

import Crawler from "crawler";

import { langs } from "./kiranico_armor.js";

interface UrlInfo {
    url: string;
    lang: string;
}

interface DecoInfo {
    name: string;
    slotSize: number;
    skillName: string;
    skillLevel: number;
    text: string;
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

    const baseDir = path.join("temp_data", "deco");

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            console.log();

            fs.ensureDirSync(baseDir);

            const proms = [] as Promise<void>[];

            for (const lang of langs) {
                const infos = allInfos[lang];

                const filename = path.join(baseDir, `deco.${lang}.json`);
                const dataStr = JSON.stringify(infos, null, 4);

                const prom = new Promise<void>((resolve, reject) => {
                    fs.writeFile(filename, dataStr, (err) => {
                        if (err) {
                            reject(err);
                        } else {
                            console.log(
                                `Kiranico ${lang} deco data file write done`
                            );
                            resolve();
                        }
                    });
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
