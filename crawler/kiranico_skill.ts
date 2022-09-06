import fs from "fs-extra";
import path from "path";

import Crawler from "crawler";

import { langs } from "./kiranico_armor.js";
import { makeId } from "./util.js";

interface UrlInfo {
    url: string;
    lang: string;
}

interface SkillInfo {
    name: string;
    text: string;
}

interface FinalSkillInfo {
    id: string;
    names: { [key: string]: string };
    texts: { [key: string]: string };
}

const allInfos: { [key: string]: SkillInfo[] } = {};

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

    const baseDir = path.join("temp_data");

    return new Promise<void>((resolve, reject) => {
        c.on("drain", () => {
            console.log();

            fs.ensureDirSync(baseDir);

            const finalInfos = [] as FinalSkillInfo[];

            const enInfos = allInfos["en"];

            enInfos.forEach((enInfo, index) => {
                const names = {} as { [key: string]: string };
                const texts = {} as { [key: string]: string };

                langs.forEach((lang) => {
                    const langInfo = allInfos[lang][index];

                    names[lang] = langInfo.name;
                    texts[lang] = langInfo.text;
                });

                const finalInfo = {
                    id: makeId(enInfo.name),
                    names,
                    texts,
                } as FinalSkillInfo;

                finalInfos.push(finalInfo);
            });

            const filename = path.join(baseDir, "skill.json");
            const dataStr = JSON.stringify(finalInfos, null, 4);

            fs.writeFile(filename, dataStr, (err) => {
                if (err) {
                    return reject(err);
                }

                console.log(`Kiranico skill data file write done`);
                resolve();
            });
        });
    });
}
