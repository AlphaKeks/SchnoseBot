import { SlashCommandBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";
import { KZMap } from "../../lib/types/gokz";
import * as g from "../../lib/functions/gokz";

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

	async execute(interaction: CommandInteraction) {
		interaction.deferReply();

		const tier = interaction.options.get("tier")?.value || null;
		const globalMaps = await g.getMapsAPI();
		if (!globalMaps.success)
			return reply(interaction, { content: globalMaps.error });

		const maps: KZMap[] = [];

		if (tier) {
			globalMaps.data!.forEach((x) => {
				if (x.difficulty === tier) maps.push(x);
			});

			const map = maps[Math.floor(Math.random() * maps.length)];
			return reply(interaction, {
				content: `🎲 \`${map.name} (T${map.difficulty})\``,
			});
		} else {
			const map =
				globalMaps.data![Math.floor(Math.random() * globalMaps.data!.length)];
			return reply(interaction, {
				content: `🎲 \`${map.name} (T${map.difficulty})\``,
			});
		}
	},
};
