import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getTimes, getFilterDist, getMaps } from "gokz.js";
import { validateTarget } from "../lib/functions/schnose";
import { reply } from "../lib/functions/discord";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";

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
		.addBooleanOption((o) => o.setName("runtype").setDescription("TP = true, PRO = false"))
		.addStringOption((o) => o.setName("target").setDescription("Specify a target")),
	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputTier = interaction.options.getInteger("tier") || null;
		const inputMode = interaction.options.getString("mode") || null;
		const inputRuntype = interaction.options.getBoolean("runtype") || false;
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

		let modeID: number;
		switch (mode) {
			case "kz_timer":
				modeID = 200;
				break;
			case "kz_simple":
				modeID = 201;
				break;
			case "kz_vanilla":
				modeID = 202;
				break;
			default:
				modeID = 200;
		}

		const doableMaps = await getFilterDist(modeID, inputRuntype);
		if (!doableMaps.success) return reply(interaction, { content: doableMaps.error });

		const completedMaps = await getTimes(targetValidation.data!.value!, mode, inputRuntype);
		if (!completedMaps.success) return reply(interaction, { content: completedMaps.error });

		const compIDs: number[] = [];
		completedMaps.data!.forEach((map) => compIDs.push(map.map_id));

		const uncompIDs: number[] = [];
		doableMaps.data!.forEach((map) => {
			if (!compIDs.includes(map.map_id)) uncompIDs.push(map.map_id);
		});

		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const uncompletedMaps: string[] = [];
		globalMaps.data!.forEach((map) => {
			if (
				uncompIDs.includes(map.id) &&
				(inputTier ? map.difficulty === inputTier : true) &&
				(inputRuntype ? !map.name.startsWith("kzpro_") : true)
			)
				uncompletedMaps.push(map.name);
		});

		if (uncompletedMaps.length === 0)
			return reply(interaction, {
				content: "Congrats! You have no maps left to complete, good job 🎉"
			});

		let text = ``;
		for (let i = 0; i < uncompletedMaps.length; i++) {
			if (i === 10) {
				text += `...${uncompletedMaps.length - 10} more`;
				break;
			}
			text += `> ${uncompletedMaps[i]}\n`;
		}

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`Uncompleted Maps - ${modeMap.get(mode)} ${inputRuntype ? "TP" : "PRO"} ${
					inputTier ? `[T${inputTier}]` : ""
				}`
			)
			.setDescription(text)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: client.icon });

		return reply(interaction, { embeds: [embed] });
	}
};
