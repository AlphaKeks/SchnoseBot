import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getPlace } from "gokz.js";
import { validateTarget } from "../lib/functions/schnose";
import { parseTime } from "../lib/functions/util";
import { reply } from "../lib/functions/discord";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";
import { recent_wasm } from "../../rust/pkg/gokz_wasm";
import * as W from "src/lib/types/wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("recent")
		.setDescription("Get a player's most recent personal best.")
		.addStringOption((o) =>
			o.setName("target").setDescription("Specify a player.").setRequired(false)
		),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputTarget = interaction.options.getString("target") || null;

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		const request = await recent_wasm(targetValidation.data!.value!);

		let result;
		try {
			result = JSON.parse(request) as W.recent_wasm;
		} catch (_) {
			return reply(interaction, { content: request });
		}

		if (!result?.map_name) return reply(interaction, { content: request });

		const place = await getPlace(result);
		const timestamp = Date.parse(result.created_on);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${result!.player_name} on ${result!.map_name}`)
			.setURL(`https://kzgo.eu/maps/${result!.map_name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					result!.map_name
				}.jpg`
			)
			.addFields([
				{
					name: `${modeMap.get(result.mode)}`,
					value: `${result!.teleports > 0 ? "TP" : "PRO"}: ${
						result?.time ? parseTime(result.time) : "😔"
					} (#${place && place.success ? `${place?.data}` : `${result ? "?" : ""}`})

				> <t:${parseInt((timestamp / 1000).toString())}:R>`,
					inline: true
				}
			])
			.setTimestamp()
			.setFooter({ text: `(͡ ͡° ͜ つ ͡͡°)7`, iconURL: client.icon });

		return reply(interaction, { embeds: [embed] });
	}
};
