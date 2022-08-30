import Crawler from "crawler";

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

            done();
        },
    });
}
