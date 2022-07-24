import { z } from "zod";

export type response = { success: boolean; data?: any; error?: any };

const modeMap = new Map();
modeMap.set("kz_timer", "KZTimer");
modeMap.set("kz_simple", "SimpleKZ");
modeMap.set("kz_vanilla", "Vanilla");
modeMap.set("none", "ALL");
export default modeMap;

export const KZMap = z.object({
	id: z.number(),
	name: z.string(),
	filesize: z.union([z.number(), z.bigint()]),
	validated: z.boolean(),
	difficulty: z.number(),
	created_on: z.string(),
	updated_on: z.string(),
	approved_by_steamid64: z.string(),
	workshop_url: z.string(),
	download_url: z.string().optional(),
});
export type KZMap = z.infer<typeof KZMap>;

export const Player = z.object({
	steamid64: z.string(),
	steam_id: z.string(),
	is_banned: z.boolean(),
	total_records: z.number(),
	name: z.string(),
});
export type Player = z.infer<typeof Player>;

export const Record = z.object({
	id: z.union([z.number(), z.bigint()]),
	steamid64: z.string(),
	player_name: z.string(),
	steam_id: z.string(),
	server_id: z.number(),
	map_id: z.number(),
	stage: z.number(),
	mode: z.string(),
	tickrate: z.number(),
	time: z.number(),
	teleports: z.number(),
	created_on: z.string(),
	updated_on: z.string(),
	updated_by: z.union([z.number(), z.bigint()]),
	record_filter_id: z.number(),
	server_name: z.string(),
	map_name: z.string(),
	points: z.number(),
	replay_id: z.union([z.number(), z.bigint()]),
});
export type Record = z.infer<typeof Record>;
