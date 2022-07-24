import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";
import userSchema from "../../lib/schemas/user";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("mode")
		.setDescription("Save your preferred gamemode in the database.")
		.addStringOption((o) =>
			o
				.setName("mode")
				.setDescription("Choose your preferred mode.")
				.setRequired(true)
				.addChoices({ name: "KZT", value: "kz_timer" })
				.addChoices({ name: "SKZ", value: "kz_simple" })
				.addChoices({ name: "VNL", value: "kz_vanilla" })
				.addChoices({ name: "ALL", value: "none" })
		),

	async execute(interaction: CommandInteraction) {
		const mode = interaction.options.get("mode")?.value;

		const userDB = await userSchema.find({ discordID: interaction.user.id });
		if (!userDB[0]) {
			await userSchema
				.create({
					name: interaction.user.username,
					discordID: interaction.user.id,
					steamID: null,
					mode: mode,
				})
				.then(() => {
					reply(interaction, {
						content: `Successfully set mode for ${interaction.user.username}.`,
					});
				})
				.catch((e: unknown) => {
					console.error(e);
					reply(interaction, { content: "Database Error." });
				});
		} else {
			await userSchema.findOneAndUpdate(
				{ discordID: interaction.user.id },
				{ mode: mode }
			);
			reply(interaction, {
				content: `Successfully updated mode for ${interaction.user.username}.`,
			});
		}
	},
};
