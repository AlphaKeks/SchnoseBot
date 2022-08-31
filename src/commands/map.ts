import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import SchnoseBot from "src/classes/Schnose";
import { reply } from "../lib/functions/discord";
import wasm from "../../rust/pkg/gokz_wasm.js";

export default {
	data: new SlashCommandBuilder()
		.setName("map")
		.setDescription("Get detailed information on a map.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true)),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();
		const inputMap = interaction.options.getString("map")!;

		const data = JSON.parse(await wasm.get_map(inputMap)) as any;
		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(data.title)
			.setURL(data.url)
			.setThumbnail(data.thumbnail)
			.setDescription(
				`🢂 API Tier: ${data.tier}
		🢂 Mapper(s): ${data.mappers.join(", ")}
		🢂 Bonuses: ${data.bonuses}
		🢂 Global Date: ${data.date}

		🢂 Filters:
		`
			)
			.addFields(data.filters)
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu",
				iconURL: client.icon
			});

		return reply(interaction, { embeds: [embed] });
	}
};
