import fs from "fs";
import path from "path";

import Crawler from "crawler";

interface InvenArmorInfo {
    name: string;
    part: string;
}

export function parse() {
    const c = new Crawler({ rateLimit: 1000 });
    const url = "https://mhf.inven.co.kr/dataninfo/mhr/armor";

    c.queue({
        uri: url,
        callback: (err, res, done) => {
            if (err) {
                console.error(err);
                return done();
            }

            const $ = res.$;

            const infos = [] as InvenArmorInfo[];

            const table = $("#mhrDb .mhr.board.board01 table tbody tr");

            table.each((i, elem) => {
                const cols = $(elem).children("td");

                const name = $(cols[0])
                    .children("a")
                    .children("b")
                    .children("span")
                    .text();

                const part = $(cols[1]).text().trim();

                const info = { name, part } as InvenArmorInfo;

                infos.push(info);
            });

            const resultStr = JSON.stringify(infos, null, 4);

            fs.writeFile(
                path.join("data", `inven_data.json`),
                resultStr,
                (err) => {
                    if (err) {
                        console.error(err);
                    }

                    console.log("All data parsing done!");
                }
            );

            done();
        },
    });
}
