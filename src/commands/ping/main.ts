import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder().setName("ping").setDescription("pong!"),

	async execute(interaction: CommandInteraction) {
		return reply(interaction, {
			content: `pong! \`[${Date.now() - interaction.createdTimestamp}ms]\``,
		});
	},
};
