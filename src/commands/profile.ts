import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getSteamAvatar, validateTarget } from "../lib/functions/schnose";
import { reply } from "../lib/functions/discord";
import userSchema from "../lib/schemas/user";
import SchnoseBot from "src/classes/Schnose";
import { profile_wasm } from "../../rust/pkg/gokz_wasm.js";
import * as W from "src/lib/types/wasm";
import modeMap from "gokz.js/lib/api";

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

		const request = await profile_wasm(targetValidation.data!.value!, mode);

		let result;
		try {
			result = JSON.parse(request) as W.profile_wasm;
		} catch (_) {
			return reply(interaction, { content: request });
		}

		if (!result?.name) return reply(interaction, { content: request });

		let profileMode;
		const playerDB = await userSchema.find({ steamID: result.steam_id });
		if (!playerDB[0]?.mode) profileMode = "unknown";
		else profileMode = modeMap.get(playerDB[0].mode);

		/* eslint-disable no-irregular-whitespace */
		/* eslint-disable indent */
		const text = `
───────────────────────────
                      TP                                               PRO
           \`${result.tp_runs![7]}/${result.doable[0][7]} (${result.tp_perc.toFixed(
			2
		)}%)\`                    \`${result.pro_runs![7]}/${
			result.doable[1][7]
		} (${result.pro_perc.toFixed(2)}%)\`
T1     ⌠ ${result.bars[0][0]} ⌡          ⌠ ${result.bars[1][0]} ⌡
T2   ⌠ ${result.bars[0][1]} ⌡          ⌠ ${result.bars[1][1]} ⌡
T3   ⌠ ${result.bars[0][2]} ⌡          ⌠ ${result.bars[1][2]} ⌡
T4  ⌠ ${result.bars[0][3]} ⌡          ⌠ ${result.bars[1][3]} ⌡
T5   ⌠ ${result.bars[0][4]} ⌡          ⌠ ${result.bars[1][4]} ⌡
T6  ⌠ ${result.bars[0][5]} ⌡          ⌠ ${result.bars[1][5]} ⌡
T7   ⌠ ${result.bars[0][6]} ⌡          ⌠ ${result.bars[1][6]} ⌡
Records: \`${result.tp_recs}\` | \`${result.pro_recs}\`
Points: \`${result.tp_points}\` | \`${result.pro_points}\`
───────────────────────────
Rank: **${result.rank}** (${result.tp_points! + result.pro_points!})
Preferred Mode: ${profileMode}
steamID: ${result.steam_id}
		`;
		/* eslint-enable indent */
		/* eslint-enable no-irregular-whitespace */

		const avatar = await getSteamAvatar(result.steamid64);
		if (!avatar.success) return reply(interaction, { content: avatar.error });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${result.name}'s ${modeMap.get(mode)} Profile`)
			.setURL(`https://kzgo.eu/players/${result.steam_id}?${modeMap.get(mode).toLowerCase()}=`)
			.setThumbnail(avatar.data!)
			.setDescription(text)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu", iconURL: client.icon });

		return reply(interaction, { embeds: [embed] });
	}
};
