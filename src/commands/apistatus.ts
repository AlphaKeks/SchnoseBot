import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import SchnoseBot from "src/classes/Schnose";
import { reply } from "../lib/functions/discord";
import { apistatus_wasm } from "../../rust/pkg/gokz_wasm";
import * as W from "src/lib/types/wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("apistatus")
		.setDescription("Check the GlobalAPI Status."),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const request = await apistatus_wasm();

		let result;
		try {
			result = JSON.parse(request) as W.apistatus_wasm;
		} catch (_) {
			return reply(interaction, { content: request });
		}

		const statusEmbed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${result.status}`)
			.setThumbnail(
				"https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png"
			)
			.addFields([
				{
					name: "Frontend",
					value: `${result.frontend}`,
					inline: true
				},
				{
					name: "Backend",
					value: `${result.backend}`,
					inline: true
				}
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: client.icon
			});

		return reply(interaction, { embeds: [statusEmbed] });
	}
};
