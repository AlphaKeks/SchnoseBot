import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import userSchema from "../lib/schemas/user";
import { reply } from "../lib/functions/discord";
import "dotenv/config";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("db")
		.setDescription("Check your current database entries."),

	async execute(interaction: ChatInputCommandInteraction) {
		const userDB = await userSchema.find({ discordID: interaction.user.id });
		if (!userDB[0])
			return reply(interaction, {
				content: "You don't have any database entries yet."
			});

		let [userID, steamID, mode]: string[] = [
			userDB[0].discordID!,
			userDB[0].steamID || "none",
			userDB[0].mode || "none"
		];

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle("Your current database entries:")
			.setDescription(
				`> userID: ${userID}
		> steamID: ${steamID}
		> mode: ${mode}`
			)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: process.env.ICON });

		return reply(interaction, { embeds: [embed] });
	}
};
