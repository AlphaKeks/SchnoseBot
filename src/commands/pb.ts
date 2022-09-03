import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import { validateTarget } from "../lib/functions/schnose";
import { getPlace } from "gokz.js";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";
import { pb_wasm } from "../../rust/pkg/gokz_wasm";
import * as W from "src/lib/types/wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("pb")
		.setDescription("Check a player's personal best on a map.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true))
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
		)
		.addStringOption((o) => o.setName("target").setDescription("Specify a target.")),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputMap = interaction.options.getString("map")!;
		const inputMode = interaction.options.getString("mode") || null;
		const inputTarget = interaction.options.getString("target") || null;

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

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		const request = await pb_wasm(inputMap, mode, targetValidation.data!.value!);

		const result: W.pb_wasm[] = [null, null];
		try {
			const temp = JSON.parse(request);
			for (let i = 0; i < temp.length; i++) {
				try {
					result[i] = JSON.parse(temp[i]) as W.pb_wasm;
				} catch (_) {}
			}
		} catch (_) {
			return reply(interaction, { content: request });
		}

		let tpPlace: any = null; // eslint-disable-line
		let proPlace: any = null; // eslint-disable-line
		if (result[0]?.map_name) tpPlace = await getPlace(result[0]);
		if (result[1]?.map_name) proPlace = await getPlace(result[1]);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`[PB] ${
					result[0]?.player_name || result[1]?.player_name
						? `${result[0]?.player_name || result[1]?.player_name} on ${
								result[0]?.map_name || result[1]?.map_name // eslint-disable-line
						  }` // eslint-disable-line
						: `${result[0]?.map_name || result[1]?.map_name}`
				}`
			)
			.setURL(`https://kzgo.eu/maps/${result[0]?.map_name || result[1]?.map_name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					result[0]?.map_name || result[1]?.map_name
				}.jpg`
			)
			.addFields([
				{
					name: "TP",
					value: `${result[0]?.time ? parseTime(result[0].time) : "😔"} ${
						tpPlace && tpPlace.success ? `(#${tpPlace?.data})` : `${result[0] ? "?" : ""}`
					}`,
					inline: true
				},
				{
					name: "PRO",
					value: `${result[1]?.time ? parseTime(result[1].time) : "😔"} ${
						proPlace && proPlace.success ? `(#${proPlace?.data})` : `${result[1] ? "?" : ""}`
					}`,
					inline: true
				}
			])
			.setFooter({
				text: `(͡ ͡° ͜ つ ͡͡°)7 | Mode: ${modeMap.get(mode)}`,
				iconURL: client.icon
			});

		return reply(interaction, {
			embeds: [embed]
		});
	}
};
