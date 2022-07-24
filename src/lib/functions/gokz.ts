import axios from "axios";
import { response, KZMap, Player, Record } from "../types/gokz";

export async function getMapsAPI(): Promise<{
	success: boolean;
	data?: KZMap[];
	error?: any;
}> {
	let response: response = { success: false };
	await axios
		.get(`https://kztimerglobal.com/api/v2.0/maps?`, {
			params: {
				is_validated: true,
				limit: 9999,
			},
		})
		.then((r) => {
			return (response = { success: true, data: r.data });
		})
		.catch((e: unknown) => console.error(e));

	if (KZMap.safeParse(response.data[0]).success) return response;
	else return (response = { success: false, error: "API Error." });
}

export async function verifyMap(
	mapList: KZMap[],
	mapName: string
): Promise<{ success: boolean; data?: KZMap; error?: any }> {
	let response: response = { success: false };
	mapList.forEach((map) => {
		if (map.name.includes(mapName))
			return (response = { success: true, data: map });
	});
	if (response.success) return response;
	else
		return (response = {
			success: false,
			error: "Please enter a valid map name.",
		});
}

export async function playerSteamID(
	steamID: string
): Promise<{ success: boolean; data?: Player; error?: any }> {
	let response: response = { success: false };
	await axios
		.get(
			`https://kztimerglobal.com/api/v2.0/players/steamid/${encodeURIComponent(
				steamID
			)}`
		)
		.then((r) => {
			return (response = { success: true, data: r.data[0] });
		})
		.catch((e: unknown) => console.error(e));

	if (Player.safeParse(response.data).success) return response;
	else return (response = { success: false, error: "Invalid Input." });
}

export async function getWR(
	mapName: string,
	mode: string,
	course: number,
	runtype: boolean
): Promise<{ success: boolean; data?: Record; error?: any }> {
	let response: response = { success: false };
	await axios
		.get(`https://kztimerglobal.com/api/v2.0/records/top?`, {
			params: {
				map_name: mapName,
				tickrate: 128,
				stage: course,
				modes_list_string: mode,
				has_teleports: runtype,
				limit: 1,
			},
		})
		.then((r) => {
			return (response = { success: true, data: r.data[0] });
		})
		.catch((e: unknown) => console.error(e));

	if (Record.safeParse(response.data).success) return response;
	else return (response = { success: false, error: "API Error." });
}
