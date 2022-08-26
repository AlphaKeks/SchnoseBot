import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { reply } from "../lib/functions/discord";

export default {
	data: new SlashCommandBuilder().setName("ping").setDescription("pong!"),

	async execute(interaction: ChatInputCommandInteraction) {
		return reply(interaction, {
			content: `pong! \`[${Date.now() - interaction.createdTimestamp}ms]\``
		});
	}
};
