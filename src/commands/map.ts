import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import SchnoseBot from "src/classes/Schnose";
import { reply } from "../lib/functions/discord";
import { map_wasm } from "../../rust/pkg/gokz_wasm";
import * as W from "src/lib/types/wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("map")
		.setDescription("Get detailed information on a map.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true)),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();
		const inputMap = interaction.options.getString("map")!;

		const request = await map_wasm(inputMap);

		let result;
		try {
			result = JSON.parse(request) as W.map_wasm;
		} catch (_) {
			return reply(interaction, { content: request });
		}

		if (!result) return reply(interaction, { content: request });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(result.title)
			.setURL(result.url)
			.setThumbnail(result.thumbnail)
			.setDescription(
				`🢂 API Tier: ${result.tier}
		🢂 Mapper(s): ${result.mappers.join(", ")}
		🢂 Bonuses: ${result.bonuses}
		🢂 Global Date: ${result.date}

		🢂 Filters:
		`
			)
			.addFields(result.filters)
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu",
				iconURL: client.icon
			});

		return reply(interaction, { embeds: [embed] });
	}
};
