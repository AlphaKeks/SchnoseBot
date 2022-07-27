import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { EmbedBuilder } from "discord.js";
import { reply } from "../../lib/functions/discord";
import { timeString } from "../../lib/functions/util";
import * as g from "../../lib/functions/gokz";
import modeMap from "../../lib/types/gokz";
import { apiCall } from "./api";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("bpb")
		.setDescription("Check a player's PB on a bonus.")
		.addStringOption((o) =>
			o.setName("map").setDescription("Choose a map.").setRequired(true)
		)
		.addNumberOption((o) =>
			o.setName("course").setDescription("Choose a bonus.").setRequired(false)
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
		)
		.addStringOption((o) =>
			o.setName("target").setDescription("Specify a player.")
		),

	async execute(interaction: CommandInteraction) {
		interaction.deferReply();

		const input_map = interaction.options.get("map")!.value!.toString();
		const input_course = interaction.options.get("course")?.value || 1;
		const input_mode = interaction.options.get("mode")?.value || null;
		const input_target = interaction.options.get("target")?.value || null;

		// verify map
		const map = await g.verifyMap(input_map);
		if (!map.success) return reply(interaction, { content: map.error });

		// verify course
		if (input_course < 1)
			return reply(interaction, { content: "Please specify a valid bonus." });

		// verify mode
		const mode = await g.verifyMode(
			interaction,
			input_mode?.toString() || null
		);
		if (!mode.success) return reply(interaction, { content: mode.error });

		// verify target
		const target = await g.verifyTarget(
			interaction,
			input_target?.toString() || null
		);
		if (!target.success) return reply(interaction, { content: target.error });

		// execute api call
		const request = await apiCall(
			map.data!,
			mode.data!,
			input_course as number,
			target as { success: true; data: { type: string; value: string } }
		);

		// reply  to the user
		const playerName = await g.playerSteamID(target.data!.value);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`[PB] ${playerName.data?.name} on ${map.data!.name}`)
			.setURL(`https://kzgo.eu/maps/${map.data!.name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					map.data!.name
				}.jpg`
			)
			.setDescription(`Mode: ${modeMap.get(mode.data)}`)
			.addFields(
				{
					name: "TP",
					value: `${
						request.TP?.data?.time ? timeString(request.TP.data.time) : "none"
					}`,
					inline: true,
				},
				{
					name: "PRO",
					value: `${
						request.PRO?.data?.time ? timeString(request.PRO.data.time) : "none"
					}`,
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
