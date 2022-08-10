import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { reply } from "../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("invite")
		.setDescription("Invite the bot to your server!"),

	async execute(interaction: ChatInputCommandInteraction) {
		return reply(interaction, {
			content: `https://bot.schnose.eu/`,
			ephemeral: true
		});
	}
};
