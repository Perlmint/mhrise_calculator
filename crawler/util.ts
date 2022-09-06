const replacRegex = new RegExp(/\s+/g);

export function makeId(value: string) {
    return value.toLowerCase().replace(replacRegex, "_");
}
