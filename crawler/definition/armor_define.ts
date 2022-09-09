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
    skills: FinalSkillInfo[];
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
    id: string;
    level: number;
}
