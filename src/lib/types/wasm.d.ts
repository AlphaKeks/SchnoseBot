/* eslint-disable */
export type apistatus_wasm = {
	status: string;
	frontend: string;
	backend: string;
};

export type map_wasm = {
	title: string;
	url: string;
	thumbnail: string;
	tier: number;
	mappers: string[];
	bonuses: number;
	date: string;
	filters: {
		name: string;
		value: string;
		inline: boolean;
	};
} | null;

export type wr_wasm = {
	id: number;
	steamid64: string;
	player_name: string;
	steam_id: string;
	server_id: number;
	map_id: number;
	stage: number;
	mode: string;
	tickrate: number;
	time: number;
	teleports: number;
	created_on: string;
	updated_on: string;
	updated_by: number;
	record_filter_id: number;
	server_name: string;
	map_name: string;
	points: number;
	replay_id: number;
} | null;

export type pb_wasm = wr_wasm;
type bwr_wasm = wr_wasm;
export type bpb_wasm = bwr_wasm;
type recent_wasm = pb_wasm;
export type unfinished_wasm = string[];

export type profile_wasm = {
	tp_points: number;
	pro_points: number;
	tp_recs: number;
	pro_recs: number;
	tp_perc: number;
	pro_perc: number;
	tp_runs: number[];
	pro_runs: number[];
	steamid64: string;
	steam_id: string | undefined;
	is_banned: boolean;
	total_records: number;
	name: string | undefined;
	rank: string;
	doable: number[][];
	bars: string[][];
};
