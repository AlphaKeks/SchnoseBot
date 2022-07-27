import axios from "axios";
import { CommandInteraction, UserMention } from "discord.js";
import userSchema from "../schemas/user";
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
	mapName: string
): Promise<{ success: boolean; data?: KZMap; error?: any }> {
	let response: response = { success: false };

	const mapList = await getMapsAPI();
	if (!mapList.success)
		return (response = { success: false, error: "API Error." });

	mapList.data!.forEach((map) => {
		if (map.name.includes(mapName.toLowerCase()))
			return (response = { success: true, data: map });
	});
	if (response.success) return response;
	else
		return (response = {
			success: false,
			error: "Please enter a valid map name.",
		});
}

export async function verifyMode(
	interaction: CommandInteraction,
	inputMode: string | null
): Promise<{ success: boolean; data?: string; error?: any }> {
	let mode = "";
	let response: response = { success: false };

	if (inputMode) mode = inputMode;
	else {
		const userDB = await userSchema.find({ discordID: interaction.user.id });
		if (!userDB[0]?.mode) mode = "none";
		else mode = userDB[0].mode;
	}

	if (mode === ("none" || null)) {
		return (response = {
			success: false,
			error: "Please specify a mode or set a default one with `/mode`.",
		});
	} else return (response = { success: true, data: mode });
}

export async function verifyTarget(
	interaction: CommandInteraction,
	inputTarget: null | string | UserMention
): Promise<{
	success: boolean;
	data?: { type: string; value: string };
	error?: any;
}> {
	let response: response = { success: false };
	let type: string | null = null;
	if (!inputTarget) {
		const userDB = await userSchema.find({ discordID: interaction.user.id });
		if (!userDB[0]?.steamID) type = null;
		else {
			type = "steamID";
			inputTarget = userDB[0].steamID;
		}
	} else {
		if (inputTarget.startsWith("<@") && inputTarget.endsWith(">"))
			type = "mention";
		else if (/STEAM_[0-1]:[0-1]:[0-9]+/i.test(inputTarget)) type = "steamID";
		else type = "name";
	}

	switch (type) {
		case null:
			response = {
				success: false,
				error:
					"You did not specify a target and you also haven't saved a steamID in the database. You can do that with `/setsteam`, so you don't have to specify a target everytime you use a command.",
			};
			break;
		case "mention": {
			inputTarget = inputTarget!.slice(2, -1);
			if (inputTarget.startsWith("!")) inputTarget = inputTarget.slice(1);
			const userDB = await userSchema.find({ discordID: inputTarget });
			if (!userDB[0]?.steamID)
				response = {
					success: false,
					error:
						"The user you mentioned did not register a steamID in the database.",
				};
			else
				response = {
					success: true,
					data: {
						type: "steamID",
						value: userDB[0].steamID,
					},
				};
			break;
		}
		case "name": {
			const player = await playerName(inputTarget!);
			response = player;
			response.data = { type: "name", value: response.data?.steam_id };
			break;
		}
		case "steamID": {
			const player = await playerSteamID(inputTarget!);
			response = player;
			response.data = { type: "steamID", value: response.data?.steam_id };
		}
	}
	return response;
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

export async function playerName(
	name: string
): Promise<{ success: boolean; data?: Player; error?: any }> {
	let response: response = { success: false };
	await axios
		.get(`https://kztimerglobal.com/api/v2.0/players?`, {
			params: {
				name: name,
				limit: 1,
			},
		})
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

export async function getPB(
	mapName: string,
	mode: string,
	player: { success: true; data: { type: string; value: string } },
	course: number,
	runtype: boolean
): Promise<{ success: boolean; data?: Record; error?: any }> {
	let response: response = { success: false };
	await axios
		.get(`https://kztimerglobal.com/api/v2.0/records/top?`, {
			params: {
				steam_id: player.data.value,
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
