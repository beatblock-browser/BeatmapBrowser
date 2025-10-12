import { BeatMap } from "@/schema";

const mapCache = new Map<string, BeatMap>();

export function getCachedMap(id: string): BeatMap | null {
    return mapCache.get(id) || null;
}

export function setCachedMap(id: string, map: BeatMap): void {
    mapCache.set(id, map);
}

export function clearMapCache(): void {
    mapCache.clear();
}
