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
export interface GenericQueryRequest {
	query: string
}
export interface GenericQueryResult {
	query: string,
	results: BeatMap[]
}
export interface UserMapRequest {
	user_id: UserID,
	map_id: MapID
}
export interface UserRequest {
	user_id: UserID
}
