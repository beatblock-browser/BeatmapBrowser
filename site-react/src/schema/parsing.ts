// This file was auto-generated with typebinder from Rust source code. Do not change this file manually.
// Change the Rust source code instead and regenerate with typebinder.
// Rust source module: parsing

export interface LevelData {
	metadata: LevelMetadata
}
export interface LevelMetadata {
	artist: string,
	charter: string,
	difficulty: number | null,
	description: string,
	songName: string,
	artistList: string,
	bgData: BackgroundData | null,
	variants: LevelVariant[]
}
export interface BackgroundData {
	image: string,
	cyanChannel: ColorChannel | null,
	magentaChannel: ColorChannel | null,
	yellowChannel: ColorChannel | null,
	redChannel: ColorChannel | null,
	greenChannel: ColorChannel | null,
	blueChannel: ColorChannel | null
}
export interface ColorChannel {
	r: number,
	g: number,
	b: number
}
export interface LevelVariant {
	display: string,
	difficulty: number
}
