import fs from "fs";
import path from "path";

import Crawler from "crawler";

import { langs } from "./kiranico.js";

interface UrlInfo {
    url: string;
    lang: string;
}

interface SkillInfo {
    name: string;
    text: string;
}

const allInfos: { [key: string]: SkillInfo[] } = {};

async function crawlCallback(
    lang: string,
    err: Error,
    res: Crawler.CrawlerRequestResponse,
    done: () => void
) {
    const $ = res.$;

    const infos = [] as SkillInfo[];

    const skills = $("table tbody tr");

    skills.each((i, elem) => {
        const tds = $(elem).children("td");

        const skillNameElem = $(tds[0])
            .children("a")
            .children("div")
            .children("div")
            .children("p")[0];

        const skillTextElem = $(tds[1]).children("p");

        const name = $(skillNameElem).text();
        let text = "";

        skillTextElem.each((i, elem) => {
            const pText = $(elem).text();

            if (pText === "") {
                return;
            }

            text += pText;
            text += "\n";
        });

        if (text.endsWith("\n")) {
            text = text.substring(0, text.length - 1);
        }

        infos.push({ name, text });
    });

    allInfos[lang] = infos;

    console.log(`Kiranico skill parsing (lang: ${lang}) done`);

    done();
}

export async function parse() {
    console.log("Kiranico skill parsing begin");

    const c = new Crawler({ rateLimit: 1000 });

    const urlInfos = langs.map(
        (lang) =>
            ({
                url: `https://mhrise.kiranico.com/${lang}/data/skills`,
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

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            console.log();

            if (fs.existsSync("temp_data") === false) {
                fs.mkdirSync("temp_data");
            }

            const proms = [] as Promise<void>[];

            for (const lang of langs) {
                const infos = allInfos[lang];

                const filename = path.join("temp_data", `skill.${lang}.json`);
                const dataStr = JSON.stringify(infos, null, 4);

                const prom = new Promise<void>((resolve, reject) => {
                    fs.writeFile(filename, dataStr, (err) => {
                        if (err) {
                            reject(err);
                        } else {
                            console.log(
                                `Kiranico ${lang} skill data file write done`
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
