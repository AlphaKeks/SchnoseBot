import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getSteamAvatar, validateTarget } from "../lib/functions/schnose";
import { reply } from "../lib/functions/discord";
import userSchema from "../lib/schemas/user";
import { PlayerProfile } from "../lib/types/schnose";
import { getPlayer } from "gokz.js";
import modeMap, { getMaps, getTimes } from "gokz.js/lib/api";
import axios from "axios";
import SchnoseBot from "src/classes/Schnose";

export default {
	data: new SlashCommandBuilder()
		.setName("profile")
		.setDescription("Check a player's stats")
		.addStringOption((o) => o.setName("target").setDescription("Specify a target."))
		.addStringOption((o) =>
			o.setName("mode").setDescription("Specify a mode.").setChoices(
				{
					name: "KZT",
					value: "kz_timer"
				},
				{
					name: "SKZ",
					value: "kz_simple"
				},
				{
					name: "VNL",
					value: "kz_vanilla"
				}
			)
		),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputTarget = interaction.options.getString("target") || null;
		const inputMode = interaction.options.getString("mode") || null;

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		let mode: string;
		if (inputMode) mode = inputMode;
		else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]?.mode)
				return reply(interaction, {
					content: "You must either specify a mode or set a default value using `/mode`."
				});
			else mode = userDB[0].mode;
		}

		const req = await getPlayer(targetValidation.data!.value!);
		if (!req.success) return reply(interaction, { content: req.error });

		const player: PlayerProfile = {
			tpPoints: 0,
			proPoints: 0,
			tpRecords: 0,
			tpPerc: 0,
			proRecords: 0,
			proPerc: 0,
			tpFinishes: [0, 0, 0, 0, 0, 0, 0, 0],
			proFinishes: [0, 0, 0, 0, 0, 0, 0, 0],
			...req.data!
		};

		const playerDB = await userSchema.find({ steamID: player.steam_id });
		if (!playerDB[0]?.mode) player.mode = "unknown";
		else player.mode = modeMap.get(playerDB[0].mode);

		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const tiers: Map<string, number>[] = [new Map(), new Map()];
		for (let i = 0; i < globalMaps.data!.length; i++) {
			tiers[0].set(globalMaps.data![i].name, globalMaps.data![i].difficulty);
			tiers[1].set(globalMaps.data![i].name, globalMaps.data![i].difficulty);
		}

		const [tpTimes, proTimes] = await Promise.all([
			getTimes(player.steam_id, mode, true),
			getTimes(player.steam_id, mode, false)
		]);

		if (!tpTimes.success && !proTimes.success)
			return reply(interaction, { content: tpTimes.error || proTimes.error || "API Error" });

		for (
			let i = 0;
			i <
			(tpTimes.data!.length > proTimes.data!.length ? tpTimes.data!.length : proTimes.data!.length);
			i++
		) {
			if (tiers[0].has(tpTimes.data![i]?.map_name)) {
				if (tpTimes.data![i]) {
					player.tpPoints! += tpTimes.data![i].points;
					player.tpFinishes![7]++;

					switch (tiers[0].get(tpTimes.data![i].map_name)) {
						case 1:
							player.tpFinishes![0]++;
							break;
						case 2:
							player.tpFinishes![1]++;
							break;
						case 3:
							player.tpFinishes![2]++;
							break;
						case 4:
							player.tpFinishes![3]++;
							break;
						case 5:
							player.tpFinishes![4]++;
							break;
						case 6:
							player.tpFinishes![5]++;
							break;
						case 7:
							player.tpFinishes![6]++;
					}

					if (tpTimes.data![i].points === 1000) player.tpRecords!++;

					tiers[0].delete(tpTimes.data![i].map_name);
				}
			}

			if (tiers[1].has(proTimes.data![i]?.map_name)) {
				if (proTimes.data![i]) {
					player.proPoints! += proTimes.data![i].points;
					player.proFinishes![7]++;

					switch (tiers[1].get(proTimes.data![i].map_name)) {
						case 1:
							player.proFinishes![0]++;
							break;
						case 2:
							player.proFinishes![1]++;
							break;
						case 3:
							player.proFinishes![2]++;
							break;
						case 4:
							player.proFinishes![3]++;
							break;
						case 5:
							player.proFinishes![4]++;
							break;
						case 6:
							player.proFinishes![5]++;
							break;
						case 7:
							player.proFinishes![6]++;
					}

					if (proTimes.data![i].points === 1000) player.proRecords!++;

					tiers[1].delete(proTimes.data![i].map_name);
				}
			}
		}

		switch (mode) {
			case "kz_timer": {
				if (player.tpPoints! + player.proPoints! >= 1000000) player.rank = "Legend";
				else if (player.tpPoints! + player.proPoints! >= 800000) player.rank = "Master";
				else if (player.tpPoints! + player.proPoints! >= 600000) player.rank = "Pro";
				else if (player.tpPoints! + player.proPoints! >= 400000) player.rank = "Semipro";
				else if (player.tpPoints! + player.proPoints! >= 250000) player.rank = "Expert+";
				else if (player.tpPoints! + player.proPoints! >= 230000) player.rank = "Expert";
				else if (player.tpPoints! + player.proPoints! >= 200000) player.rank = "Expert-";
				else if (player.tpPoints! + player.proPoints! >= 150000) player.rank = "Skilled+";
				else if (player.tpPoints! + player.proPoints! >= 120000) player.rank = "Skilled";
				else if (player.tpPoints! + player.proPoints! >= 100000) player.rank = "Skilled-";
				else if (player.tpPoints! + player.proPoints! >= 80000) player.rank = "Regular+";
				else if (player.tpPoints! + player.proPoints! >= 70000) player.rank = "Regular";
				else if (player.tpPoints! + player.proPoints! >= 60000) player.rank = "Regular-";
				else if (player.tpPoints! + player.proPoints! >= 40000) player.rank = "Casual+";
				else if (player.tpPoints! + player.proPoints! >= 30000) player.rank = "Casual";
				else if (player.tpPoints! + player.proPoints! >= 20000) player.rank = "Casual-";
				else if (player.tpPoints! + player.proPoints! >= 10000) player.rank = "Amateur+";
				else if (player.tpPoints! + player.proPoints! >= 5000) player.rank = "Amateur";
				else if (player.tpPoints! + player.proPoints! >= 2000) player.rank = "Amateur-";
				else if (player.tpPoints! + player.proPoints! >= 1000) player.rank = "Beginner+";
				else if (player.tpPoints! + player.proPoints! >= 500) player.rank = "Beginner";
				else if (player.tpPoints! + player.proPoints! > 0) player.rank = "Beginner-";
				else player.rank = "New";
				break;
			}

			case "kz_simple": {
				if (player.tpPoints! + player.proPoints! >= 800000) player.rank = "Legend";
				else if (player.tpPoints! + player.proPoints! >= 500000) player.rank = "Master";
				else if (player.tpPoints! + player.proPoints! >= 400000) player.rank = "Pro";
				else if (player.tpPoints! + player.proPoints! >= 300000) player.rank = "Semipro";
				else if (player.tpPoints! + player.proPoints! >= 250000) player.rank = "Expert+";
				else if (player.tpPoints! + player.proPoints! >= 230000) player.rank = "Expert";
				else if (player.tpPoints! + player.proPoints! >= 200000) player.rank = "Expert-";
				else if (player.tpPoints! + player.proPoints! >= 150000) player.rank = "Skilled+";
				else if (player.tpPoints! + player.proPoints! >= 120000) player.rank = "Skilled";
				else if (player.tpPoints! + player.proPoints! >= 100000) player.rank = "Skilled-";
				else if (player.tpPoints! + player.proPoints! >= 80000) player.rank = "Regular+";
				else if (player.tpPoints! + player.proPoints! >= 70000) player.rank = "Regular";
				else if (player.tpPoints! + player.proPoints! >= 60000) player.rank = "Regular-";
				else if (player.tpPoints! + player.proPoints! >= 40000) player.rank = "Casual+";
				else if (player.tpPoints! + player.proPoints! >= 30000) player.rank = "Casual";
				else if (player.tpPoints! + player.proPoints! >= 20000) player.rank = "Casual-";
				else if (player.tpPoints! + player.proPoints! >= 10000) player.rank = "Amateur+";
				else if (player.tpPoints! + player.proPoints! >= 5000) player.rank = "Amateur";
				else if (player.tpPoints! + player.proPoints! >= 2000) player.rank = "Amateur-";
				else if (player.tpPoints! + player.proPoints! >= 1000) player.rank = "Beginner+";
				else if (player.tpPoints! + player.proPoints! >= 500) player.rank = "Beginner";
				else if (player.tpPoints! + player.proPoints! > 0) player.rank = "Beginner-";
				else player.rank = "New";
				break;
			}

			case "kz_vanilla": {
				if (player.tpPoints! + player.proPoints! >= 600000) player.rank = "Legend";
				else if (player.tpPoints! + player.proPoints! >= 400000) player.rank = "Master";
				else if (player.tpPoints! + player.proPoints! >= 300000) player.rank = "Pro";
				else if (player.tpPoints! + player.proPoints! >= 250000) player.rank = "Semipro";
				else if (player.tpPoints! + player.proPoints! >= 200000) player.rank = "Expert+";
				else if (player.tpPoints! + player.proPoints! >= 180000) player.rank = "Expert";
				else if (player.tpPoints! + player.proPoints! >= 160000) player.rank = "Expert-";
				else if (player.tpPoints! + player.proPoints! >= 140000) player.rank = "Skilled+";
				else if (player.tpPoints! + player.proPoints! >= 120000) player.rank = "Skilled";
				else if (player.tpPoints! + player.proPoints! >= 100000) player.rank = "Skilled-";
				else if (player.tpPoints! + player.proPoints! >= 80000) player.rank = "Regular+";
				else if (player.tpPoints! + player.proPoints! >= 70000) player.rank = "Regular";
				else if (player.tpPoints! + player.proPoints! >= 60000) player.rank = "Regular-";
				else if (player.tpPoints! + player.proPoints! >= 40000) player.rank = "Casual+";
				else if (player.tpPoints! + player.proPoints! >= 30000) player.rank = "Casual";
				else if (player.tpPoints! + player.proPoints! >= 20000) player.rank = "Casual-";
				else if (player.tpPoints! + player.proPoints! >= 10000) player.rank = "Amateur+";
				else if (player.tpPoints! + player.proPoints! >= 5000) player.rank = "Amateur";
				else if (player.tpPoints! + player.proPoints! >= 2000) player.rank = "Amateur-";
				else if (player.tpPoints! + player.proPoints! >= 1000) player.rank = "Beginner+";
				else if (player.tpPoints! + player.proPoints! >= 500) player.rank = "Beginner";
				else if (player.tpPoints! + player.proPoints! > 0) player.rank = "Beginner-";
				else player.rank = "New";
				break;
			}
		}

		const doable: {
			success: boolean;
			data?: any; // eslint-disable-line
			error?: string;
		} = await axios
			.get(`https://kzgo.eu/api/completions/${mode}`)
			.then((response) => {
				return { success: true, data: response.data };
			})
			.catch((_) => {
				return { success: false, error: "KZGO API Error" };
			});

		if (!doable.success) return reply(interaction, { content: doable.error });

		const doableCount = [
			[
				doable.data!.tp["1"],
				doable.data!.tp["2"],
				doable.data!.tp["3"],
				doable.data!.tp["4"],
				doable.data!.tp["5"],
				doable.data!.tp["6"],
				doable.data!.tp["7"],
				doable.data!.tp["total"]
			],
			[
				doable.data!.pro["1"],
				doable.data!.pro["2"],
				doable.data!.pro["3"],
				doable.data!.pro["4"],
				doable.data!.pro["5"],
				doable.data!.pro["6"],
				doable.data!.pro["7"],
				doable.data!.pro["total"]
			]
		];

		if (player.tpFinishes![7] > 0)
			player.tpPerc = Math.round(Math.floor((player.tpFinishes![7] / doableCount[0][7]) * 100));
		if (player.proFinishes![7] > 0)
			player.proPerc = Math.round(Math.floor((player.proFinishes![7] / doableCount[1][7]) * 100));

		const bars = [
			["", "", "", "", "", "", ""],
			["", "", "", "", "", "", ""]
		];

		for (let i = 0; i < 7; i++) {
			const amountOfBars = Math.round(Math.floor((player.tpFinishes![i] / doableCount[0][i]) * 10));
			for (let j = 0; j < amountOfBars; j++) {
				bars[0][i] += "█";
			}

			for (let k = 0; k < 10 - amountOfBars; k++) {
				bars[0][i] += "░";
			}
		}

		for (let i = 0; i < 7; i++) {
			const amountOfBars = Math.round(
				Math.floor((player.proFinishes![i] / doableCount[1][i]) * 10)
			);
			for (let j = 0; j < amountOfBars; j++) {
				bars[1][i] += "█";
			}

			for (let k = 0; k < 10 - amountOfBars; k++) {
				bars[1][i] += "░";
			}
		}

		/* eslint-disable no-irregular-whitespace */
		/* eslint-disable indent */
		const text = `
──────────────────────────────
                      TP                                               PRO
           \`${player.tpFinishes![7]}/${doableCount[0][7]} (${
			player.tpPerc
		}%)\`                     \`${player.proFinishes![7]}/${doableCount[1][7]} (${
			player.proPerc
		}%)\`
T1     ⌠ ${bars[0][0]} ⌡          ⌠ ${bars[1][0]} ⌡
T2   ⌠ ${bars[0][1]} ⌡          ⌠ ${bars[1][1]} ⌡
T3   ⌠ ${bars[0][2]} ⌡          ⌠ ${bars[1][2]} ⌡
T4  ⌠ ${bars[0][3]} ⌡          ⌠ ${bars[1][3]} ⌡
T5   ⌠ ${bars[0][4]} ⌡          ⌠ ${bars[1][4]} ⌡
T6  ⌠ ${bars[0][5]} ⌡          ⌠ ${bars[1][5]} ⌡
T7   ⌠ ${bars[0][6]} ⌡          ⌠ ${bars[1][6]} ⌡

Records: \`${player.tpRecords}\` / \`${player.proRecords}\`
Points: \`${player.tpPoints}\` / \`${player.proPoints}\`

──────────────────────────────
Rank: **${player.rank}** (${player.tpPoints! + player.proPoints!})
Preferred Mode: ${player.mode}
steamID: ${player.steam_id}
		`;
		/* eslint-enable indent */
		/* eslint-enable no-irregular-whitespace */

		const avatar = await getSteamAvatar(player.steamid64);
		if (!avatar.success) return reply(interaction, { content: avatar.error });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${player.name}'s ${modeMap.get(mode)} Profile`)
			.setURL(`https://kzgo.eu/players/${player.steam_id}?${modeMap.get(mode).toLowerCase()}=`)
			.setThumbnail(avatar.data!)
			.setDescription(text)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu", iconURL: client.icon });

		return reply(interaction, { embeds: [embed] });
	}
};
