import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { validateTarget } from "../lib/functions/schnose";
import { reply } from "../lib/functions/discord";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";
import { unfinished_wasm } from "../../rust/pkg/gokz_wasm";

export default {
	data: new SlashCommandBuilder()
		.setName("unfinished")
		.setDescription("Check which maps you still need to complete.")
		.addIntegerOption((o) =>
			o
				.setName("tier")
				.setDescription("Filter for a specific tier.")
				.addChoices({ name: "1 (Very Easy)", value: 1 })
				.addChoices({ name: "2 (Easy)", value: 2 })
				.addChoices({ name: "3 (Medium)", value: 3 })
				.addChoices({ name: "4 (Hard)", value: 4 })
				.addChoices({ name: "5 (Very Hard)", value: 5 })
				.addChoices({ name: "6 (Extreme)", value: 6 })
				.addChoices({ name: "7 (Death)", value: 7 })
		)
		.addStringOption((o) =>
			o
				.setName("mode")
				.setDescription("Specify a mode.")
				.addChoices({ name: "KZTimer", value: "kz_timer" })
				.addChoices({ name: "SimpleKZ", value: "kz_simple" })
				.addChoices({ name: "Vanilla", value: "kz_vanilla" })
		)
		.addStringOption((o) =>
			o
				.setName("runtype")
				.setDescription("TP/PRO")
				.addChoices({ name: "TP", value: "true" })
				.addChoices({ name: "PRO", value: "false" })
		)
		.addStringOption((o) => o.setName("target").setDescription("Specify a target")),
	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputTier = interaction.options.getInteger("tier") || undefined;
		const inputMode = interaction.options.getString("mode") || null;
		const inputRuntype = interaction.options.getString("runtype")
			? interaction.options.getString("runtype") === "true"
				? true
				: false
			: false;
		const inputTarget = interaction.options.getString("target") || null;

		/* eslint-disable indent */
		/* eslint-disable no-mixed-spaces-and-tabs */
		const mode = inputMode
			? inputMode
			: await (async () => {
					const userDB = await userSchema.find({
						discordID: interaction.user.id
					});
					if (!userDB[0]?.mode) return null;
					else return userDB[0].mode;
			  })();
		/* eslint-enable no-mixed-spaces-and-tabs */
		/* eslint-enable indent */
		if (!mode)
			return reply(interaction, {
				content: "You must either specify a mode or set a default option with `/mode`."
			});

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		const request = await unfinished_wasm(
			inputTier,
			mode,
			inputRuntype,
			targetValidation.data!.value!
		);

		if (!request.startsWith(">") || !request.startsWith("Congrats"))
			return reply(interaction, { content: request });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`Uncompleted Maps - ${modeMap.get(mode)} ${inputRuntype ? "TP" : "PRO"} ${
					inputTier ? `[T${inputTier}]` : ""
				}`
			)
			.setDescription(request)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: client.icon });

		return reply(interaction, { embeds: [embed] });
	}
};
