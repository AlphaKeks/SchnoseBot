import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { EmbedBuilder } from "discord.js";
import { reply } from "../../lib/functions/discord";
import * as g from "../../lib/functions/gokz";
import { timeString } from "../../lib/functions/util";
import modeMap from "../../lib/types/gokz";
import userSchema from "../../lib/schemas/user";
import "dotenv/config";
import { apiCall } from "./api";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("wr")
		.setDescription("Check the World Record on a map.")
		.addStringOption((o) =>
			o.setName("map").setDescription("Choose a map.").setRequired(true)
		)
		.addStringOption((o) =>
			o
				.setName("mode")
				.setDescription("Choose a mode.")
				.setRequired(false)
				.addChoices({ name: "KZT", value: "kz_timer" })
				.addChoices({ name: "SKZ", value: "kz_simple" })
				.addChoices({ name: "VNL", value: "kz_vanilla" })
				.addChoices({ name: "ALL", value: "none" })
		),

	async execute(interaction: CommandInteraction) {
		interaction.deferReply();

		const input_map = interaction.options.get("map")!.value!.toString()!;
		const input_mode = interaction.options.get("mode")?.value || null;

		// verify map
		const globalMaps = await g.getMapsAPI();
		if (!globalMaps.success)
			return reply(interaction, { content: globalMaps.error });

		const map = await g.verifyMap(globalMaps.data!, input_map);
		if (!map.success) return reply(interaction, { content: map.error });

		// verify mode
		let mode = "";
		if (input_mode) {
			mode = input_mode.toString();
		} else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]?.mode) mode = "none";
			else mode = userDB[0].mode;
		}
		if (mode === ("none" || null))
			return reply(interaction, {
				content: "Please specify a mode or set a default one with `/mode`.",
			});

		// execute api call
		const request = await apiCall(map.data!, mode);

		// reply to the user
		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`[WR] ${map.data!.name}`)
			.setURL(`https://kzgo.eu/maps/${map.data!.name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					map.data!.name
				}.jpg`
			)
			.setDescription(`Mode: ${modeMap.get(mode)}`)
			.addFields(
				{
					name: "TP",
					value: `${
						request.TP?.data?.time ? timeString(request.TP.data.time) : ""
					}\n (${request.TP?.data?.player_name || ""})`,
					inline: true,
				},
				{
					name: "PRO",
					value: `${
						request.PRO?.data?.time ? timeString(request.PRO.data.time) : ""
					}\n (${request.PRO?.data?.player_name || ""})`,
					inline: true,
				}
			)
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: process.env.ICON,
			});

		reply(interaction, { embeds: [embed] });
	},
};
