import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import { getMaps } from "gokz.js";
import { Map } from "gokz.js/lib/types";
import { reply } from "../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("random")
		.setDescription("Get a random KZ map. You can sort by tier if you want :)")
		.addIntegerOption((o) =>
			o
				.setName("tier")
				.setDescription("Filter for a specific tier.")
				.setRequired(false)
				.addChoices({ name: "1 (Very Easy)", value: 1 })
				.addChoices({ name: "2 (Easy)", value: 2 })
				.addChoices({ name: "3 (Medium)", value: 3 })
				.addChoices({ name: "4 (Hard)", value: 4 })
				.addChoices({ name: "5 (Very Hard)", value: 5 })
				.addChoices({ name: "6 (Extreme)", value: 6 })
				.addChoices({ name: "7 (Death)", value: 7 })
		),

	async execute(interaction: ChatInputCommandInteraction) {
		await interaction.deferReply();

		const tier = interaction.options.getInteger("tier") || null;
		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const maps: Map[] = [];

		if (tier) {
			globalMaps.data!.forEach((x: Map) => {
				if (x.difficulty === tier) maps.push(x);
			});

			const map = maps[Math.floor(Math.random() * maps.length)];
			return reply(interaction, {
				content: `🎲 \`${map.name} (T${map.difficulty})\``
			});
		} else {
			const map = globalMaps.data![Math.floor(Math.random() * globalMaps.data!.length)];
			return reply(interaction, {
				content: `🎲 \`${map.name} (T${map.difficulty})\``
			});
		}
	}
};
