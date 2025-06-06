// This file was auto-generated with typebinder from Rust source code. Do not change this file manually.
// Change the Rust source code instead and regenerate with typebinder.
// Rust source module: signin

import { UserID } from "@/schema";
export interface DiscordTokenRequest {
	access_token: string
}
export interface DiscordUser {
	id: string,
	username: string,
	discriminator: string,
	global_name: string,
	verified: boolean
}
export interface UserToken {
	id: UserID,
	token: string
}
