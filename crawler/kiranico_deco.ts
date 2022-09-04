import fs from "fs";
import path from "path";

import Crawler from "crawler";

import { langs } from "./kiranico.js";

interface UrlInfo {
    url: string;
    lang: string;
}

function crawlCallback(
    lang: string,
    err: Error,
    res: Crawler.CrawlerRequestResponse,
    done: () => void
) {
    done();
}

export async function parse() {
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
}
