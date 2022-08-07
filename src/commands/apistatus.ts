import {
	SlashCommandBuilder,
	ChatInputCommandInteraction,
	EmbedBuilder,
} from "discord.js";
import { APIStatus } from "gokz.js";
import { reply } from "../lib/functions/discord";
import "dotenv/config";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("apistatus")
		.setDescription("Check the GlobalAPI Status."),

	async execute(interaction: ChatInputCommandInteraction) {
		const statusRequest = await APIStatus();

		const statusEmbed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${statusRequest.status}`)
			.setThumbnail(
				"https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png"
			)
			.addFields([
				{
					name: "Frontend",
					value: `${statusRequest.frontEnd}`,
					inline: true,
				},
				{
					name: "Backend",
					value: `${statusRequest.backEnd}`,
					inline: true,
				},
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: process.env.ICON,
			});

		return reply(interaction, { embeds: [statusEmbed] });
	},
};
