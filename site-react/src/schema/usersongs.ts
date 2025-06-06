// This file was auto-generated with typebinder from Rust source code. Do not change this file manually.
// Change the Rust source code instead and regenerate with typebinder.
// Rust source module: usersongs

import { UserID, BeatMap } from "@/schema";
export interface UsersongsRequest {
	user_id: UserID
}
export interface UserpageArguments {
	user: string
}
export interface SongsResult {
	results: BeatMap[]
}
