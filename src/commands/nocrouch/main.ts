import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("nocrouch")
		.setDescription("Approximate potential distance of a nocrouch jump.")
		.addNumberOption((o) =>
			o
				.setName("distance")
				.setDescription("the distance of your nocrouch jump")
				.setRequired(true)
		)
		.addNumberOption((o) =>
			o
				.setName("max")
				.setDescription("the max speed of your nocrouch jump")
				.setRequired(true)
		),

	execute(interaction: CommandInteraction) {
		const dist = parseInt(
			interaction.options.get("distance")!.value!.toString()
		);
		const max = parseInt(interaction.options.get("max")!.value!.toString());
		const approx = dist + (max / 128) * 4;

		return reply(interaction, {
			content: `Approximated distance: \`${approx.toFixed(4)}\``,
		});
	},
};
