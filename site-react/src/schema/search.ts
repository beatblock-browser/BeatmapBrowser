// This file was auto-generated with typebinder from Rust source code. Do not change this file manually.
// Change the Rust source code instead and regenerate with typebinder.
// Rust source module: search

import { BeatMap } from "@/schema";
export interface SearchRequest {
	query: string
}
export interface SearchResult {
	query: string,
	results: BeatMap[]
}
