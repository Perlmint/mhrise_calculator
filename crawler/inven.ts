import fs from "fs";
import path from "path";

import Crawler from "crawler";

export interface InvenArmorInfo {
    name: string;
    part: string;
    sexType: SexType;
    rarity: number;
}

enum SexType {
    all = "all",
    male = "man.",
    female = "female",
}

const rarityRegex = new RegExp(/rare(\d+)/);

const rarityNameMap = {
    머리: "helm",
    몸통: "torso",
    팔: "arm",
    허리: "waist",
    다리: "feet",
} as { [key: string]: string };

const sexTypeDict = {
    여성용: SexType.female,
    남성용: SexType.male,
    "": SexType.all,
} as { [key: string]: SexType };

export async function parse() {
    const prom = new Promise<void>((resolve, reject) => {
        console.log("Inven parsing begin");

        const c = new Crawler({ rateLimit: 1000 });
        const url = "https://mhf.inven.co.kr/dataninfo/mhr/armor";

        c.queue({
            uri: url,
            callback: (err, res, done) => {
                if (err) {
                    reject(err);

                    return done();
                }

                const $ = res.$;

                const infos = [] as InvenArmorInfo[];

                const table = $("#mhrDb .mhr.board.board01 table tbody tr");

                table.each((i, elem) => {
                    const cols = $(elem).children("td");

                    const nameSpan = $(cols[0])
                        .children("a")
                        .children("b")
                        .children("span");

                    const sexSpan = $(cols[0]).children("span.sextype");

                    const name = nameSpan.text();
                    const sexType = sexTypeDict[sexSpan.text()];
                    const rarityClass = nameSpan.attr("class");

                    const rarity = Number.parseInt(
                        rarityClass?.match(rarityRegex)![1]!
                    );

                    const part = rarityNameMap[$(cols[1]).text().trim()];

                    const info = {
                        name,
                        part,
                        sexType,
                        rarity,
                    } as InvenArmorInfo;

                    infos.push(info);
                });

                console.log("Inven parsing done");

                if (fs.existsSync("temp_data") === false) {
                    fs.mkdirSync("temp_data");
                }

                const resultStr = JSON.stringify(infos, null, 4);

                fs.writeFile(
                    path.join("temp_data", `inven_data.json`),
                    resultStr,
                    (err) => {
                        if (err) {
                            reject(err);
                        } else {
                            console.log("Inven data file write done");

                            resolve();
                        }

                        done();
                    }
                );
            },
        });
    });

    return prom;
}
