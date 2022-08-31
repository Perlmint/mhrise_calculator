import { parse as kiranicoParse } from "./kiranico.js";
import { parse as invenParse } from "./inven.js";

async function main() {
    kiranicoParse();
    invenParse();
}

await main();
