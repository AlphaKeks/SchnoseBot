import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getPlace, getRecent } from "gokz.js";
import { validateTarget } from "../lib/functions/schnose";
import modeMap from "../lib/functions/schnose";
import { parseTime } from "../lib/functions/util";
import { reply } from "../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("recent")
		.setDescription("Get a player's most recent personal best.")
		.addStringOption((o) =>
			o.setName("target").setDescription("Specify a player.").setRequired(false)
		),

	async execute(interaction: ChatInputCommandInteraction) {
		await interaction.deferReply();

		const inputTarget = interaction.options.getString("target") || null;

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		const req = await getRecent(targetValidation.data!.value!);
		if (!req.success) return reply(interaction, { content: req.error });

		const place = await getPlace(req.data!);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${req.data!.player_name} on ${req.data!.map_name}`)
			.setURL(`https://kzgo.eu/maps/${req.data!.map_name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					req.data!.map_name
				}.jpg`
			)
			.addFields([
				{
					name: `${modeMap.get(req.data!.mode)}`,
					value: `${req.data!.teleports > 0 ? "TP" : "PRO"}: ${parseTime(req.data!.time)} (#${
						place?.success ? `${place?.data}` : `?`
					})

				> <t:${parseInt(req.data!.created_on) / 1000}:R>`,
					inline: true
				}
			])
			.setTimestamp()
			.setFooter({ text: `(͡ ͡° ͜ つ ͡͡°)7` });

		return reply(interaction, { embeds: [embed] });
	}
};
