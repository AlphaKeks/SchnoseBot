import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { getPlayer } from "gokz.js";
import { reply } from "../lib/functions/discord";
import userSchema from "../lib/schemas/user";

export default {
	data: new SlashCommandBuilder()
		.setName("setsteam")
		.setDescription("Save your steamID in the database.")
		.addStringOption((o) =>
			o.setName("steamid").setDescription("e.g. STEAM_1:1:161178172").setRequired(true)
		),

	async execute(interaction: ChatInputCommandInteraction) {
		const steamID = interaction.options.getString("steamid")!;

		const player = await getPlayer(steamID);
		if (!player.success) return reply(interaction, { content: player.error });
		else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]) {
				await userSchema
					.create({
						name: interaction.user.username,
						discordID: interaction.user.id,
						steamID: player.data!.steam_id,
						mode: null
					})
					.then(() => {
						reply(interaction, {
							content: `Successfully set steamID for ${player.data?.name}.`
						});
					})
					.catch((e: unknown) => {
						console.error(e);
						reply(interaction, { content: "Database Error." });
					});
			} else {
				await userSchema.findOneAndUpdate(
					{ discordID: interaction.user.id },
					{ steamID: player.data!.steam_id }
				);
				reply(interaction, {
					content: `Successfully updated steamID for ${player.data?.name}.`
				});
			}
		}
	}
};
