import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";
import * as g from "../../lib/functions/gokz";
import userSchema from "../../lib/schemas/user";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("setsteam")
		.setDescription("Save your steamID in the database.")
		.addStringOption((o) =>
			o
				.setName("steamid")
				.setDescription("e.g. STEAM_1:1:161178172")
				.setRequired(true)
		),

	async execute(interaction: CommandInteraction) {
		const steamID = interaction.options.get("steamid")?.value?.toString();

		const player = await g.playerSteamID(steamID!);
		if (!player.success) return reply(interaction, { content: player.error });
		else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]) {
				await userSchema
					.create({
						name: interaction.user.username,
						discordID: interaction.user.id,
						steamID: steamID,
						mode: null,
					})
					.then(() => {
						reply(interaction, {
							content: `Successfully set steamID for ${player.data?.name}.`,
						});
					})
					.catch((e: unknown) => {
						console.error(e);
						reply(interaction, { content: "Database Error." });
					});
			} else {
				await userSchema.findOneAndUpdate(
					{ discordID: interaction.user.id },
					{ steamID: steamID }
				);
				reply(interaction, {
					content: `Successfully updated steamID for ${player.data?.name}.`,
				});
			}
		}
	},
};
