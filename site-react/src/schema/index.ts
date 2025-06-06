// This file was auto-generated with typebinder from Rust source code. Do not change this file manually.
// Change the Rust source code instead and regenerate with typebinder.
// Rust source module: 

import { Uuid } from "@/lib/env";
import { LevelVariant } from "@/schema/parsing";
export type UserID = Uuid;
export type MapID = Uuid;
export interface BeatMap {
	song: string,
	artist: string,
	charter: string,
	charter_uid: UserID,
	difficulties: LevelVariant[],
	description: string,
	artist_list: string,
	image: boolean,
	upvotes: number,
	upload_date: string,
	update_date: string,
	id: MapID
}
export interface User {
	maps: MapID[],
	downloaded: MapID[],
	upvoted: MapID[],
	id: UserID,
	links: AccountLink[]
}
export type AccountLink = {
	"discord": number
} | {
	"google": string
};
