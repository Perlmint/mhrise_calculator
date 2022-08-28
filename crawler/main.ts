import Crawler from "crawler";

const langs = [
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

async function main() {
    console.log("Hello");

    const c = new Crawler({
        rateLimit: 1000,
        callback: (err, res, done) => {
            if (err) {
                console.error(err);
                return done();
            }

            const $ = res.$;
            const dataTable = $("table").get(0);

            console.log(dataTable);

            return done();
        },
    });

    const urls = [];

    for (const lang of langs) {
        for (let i = 0; i < maxArmorRarity; ++i) {
            const url = `https://mhrise.kiranico.com/${lang}/data/armors?view=${i}`;
            urls.push(url);

            break;
        }

        break;
    }

    c.queue(urls);
}

await main();
