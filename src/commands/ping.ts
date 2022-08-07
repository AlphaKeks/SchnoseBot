import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { reply } from "../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder().setName("ping").setDescription("pong!"),

	async execute(interaction: ChatInputCommandInteraction) {
		return reply(interaction, {
			content: `pong! \`[${Date.now() - interaction.createdTimestamp}ms]\``,
		});
	},
};
