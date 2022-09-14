export interface KiranicoArmorInfo {
    name: string;
    rarity: number;
    stat: ArmorStatInfo;
    skills: SkillInfo[];
    slots: number[];
}

export interface FinalArmorInfo {
    id: string;
    part: string;
    sexType: string;
    names: { [key: string]: string };
    rarity: number;
    stat: ArmorStatInfo;
    skills: { [key: string]: FinalSkillInfo };
    slots: number[];
}

export interface ArmorStatInfo {
    defense: number;
    fireRes: number;
    waterRes: number;
    iceRes: number;
    elecRes: number;
    dragonRes: number;
}

export interface SkillInfo {
    name: string;
    level: number;
}

export interface FinalSkillInfo {
    level: number;
}

export const ArmorParts = ["helm", "torso", "arm", "waist", "feet"];
export const ArmorSlotCount = 3;
