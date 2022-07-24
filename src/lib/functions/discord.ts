import { CommandInteraction, InteractionReplyOptions } from "discord.js";

export function reply(
	interaction: CommandInteraction,
	input: InteractionReplyOptions
) {
	if (interaction.deferred) return interaction.editReply(input);
	else return interaction.reply(input);
}
