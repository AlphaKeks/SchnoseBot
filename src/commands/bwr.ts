import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";
import { bwr_wasm } from "../../rust/pkg/gokz_wasm";
import * as W from "src/lib/types/wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("bwr")
		.setDescription("Check the World Record on a bonus.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true))
		.addIntegerOption((o) => o.setName("course").setDescription("Specify a bonus."))
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

		const inputMap = interaction.options.getString("map")!;
		const inputCourse = interaction.options.getInteger("course") || 0;
		const inputMode = interaction.options.getString("mode") || null;

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

		const request = await bwr_wasm(inputMap, inputCourse, mode);

		const result: W.bwr_wasm[] = [null, null];
		try {
			const temp = JSON.parse(request);
			for (let i = 0; i < temp.length; i++) {
				try {
					result[i] = JSON.parse(temp[i]) as W.bwr_wasm;
				} catch (_) {
					result[i] = null;
				}
			}
		} catch (_) {
			return reply(interaction, { content: request });
		}

		if (!result[0] && !result[1]) return reply(interaction, { content: request });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`[BWR ${inputCourse}] - ${result[0]?.map_name || result[1]?.map_name}`)
			.setURL(
				`https://kzgo.eu/maps/${result[0]?.map_name || result[1]?.map_name}&bonus=${inputCourse}`
			)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					result[0]?.map_name || result[1]?.map_name
				}.jpg`
			)
			.addFields([
				{
					name: "TP",
					value: `${result[0]?.time ? parseTime(result[0].time) : "😔"} ${
						result[0]?.player_name ? `(${result[0].player_name})` : ""
					}`,
					inline: true
				},
				{
					name: "PRO",
					value: `${result[1]?.time ? parseTime(result[1].time) : "😔"} ${
						result[1]?.player_name ? `(${result[1].player_name})` : ""
					}`,
					inline: true
				}
			])
			.setFooter({
				text: `(͡ ͡° ͜ つ ͡͡°)7 | Mode: ${modeMap.get(result[0]?.mode || result[1]?.mode)}`,
				iconURL: client.icon
			});

		return reply(interaction, {
			embeds: [embed]
		});
	}
};
