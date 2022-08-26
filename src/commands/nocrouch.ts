import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { reply } from "../lib/functions/discord";

export default {
	data: new SlashCommandBuilder()
		.setName("nocrouch")
		.setDescription("Approximate potential distance of a nocrouch jump.")
		.addNumberOption((o) =>
			o
				.setName("distance")
				.setDescription("Specify the distance of your nocrouch jump.")
				.setRequired(true)
		)
		.addNumberOption((o) =>
			o
				.setName("max")
				.setDescription("Specify the max speed of your nocrouch jump.")
				.setRequired(true)
		),
	execute(interaction: ChatInputCommandInteraction) {
		const distance = interaction.options.getNumber("distance")!;
		const max = interaction.options.getNumber("max")!;
		const approx = distance + (max / 128) * 4;

		return reply(interaction, {
			content: `Approximated distance: \`${approx.toFixed(4)}\``
		});
	}
};
