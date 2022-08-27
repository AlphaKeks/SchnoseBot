import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { getFilters, getMapKZGO, getMaps, validateMap } from "gokz.js";
import SchnoseBot from "src/classes/Schnose";
import { reply } from "../lib/functions/discord";

export default {
	data: new SlashCommandBuilder()
		.setName("map")
		.setDescription("Get detailed information on a map.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true)),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputMap = interaction.options.getString("map")!;

		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const mapValidation = await validateMap(inputMap, globalMaps.data!);
		if (!mapValidation.success) return reply(interaction, { content: mapValidation.error });

		const kzgoMap = await getMapKZGO(mapValidation.data!.name);
		if (!kzgoMap.success) return reply(interaction, { content: kzgoMap.error });

		const mappers: string[] = [];
		for (let i = 0; i < kzgoMap.data!.mapperIds.length; i++) {
			mappers.push(
				`[${kzgoMap.data!.mapperNames[i]}](https://steamcommunity.com/profiles/${
					kzgoMap.data!.mapperIds[i]
				})`
			);
		}

		const filters = await getFilters(mapValidation.data!.id, 0);
		if (!filters.success) return reply(interaction, { content: filters.error });

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${mapValidation.data!.name}`)
			.setURL(`https://kzgo.eu/maps/${mapValidation.data!.name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					mapValidation.data!.name
				}.jpg`
			)
			.setDescription(
				`🢂 API Tier: ${mapValidation.data!.difficulty}
		🢂 Mapper(s): ${mappers.join(", ")}
		🢂 Bonuses: ${kzgoMap.data!.bonuses}
		🢂 Global Date: <t:${Date.parse(kzgoMap.data!.date) / 1000}:d>

		🢂 Filters:
		`
			)
			.addFields([
				{
					name: filters.data!.KZT.displayMode,
					value: filters.data!.KZT.icon,
					inline: true
				},
				{
					name: filters.data!.SKZ.displayMode,
					value: filters.data!.SKZ.icon,
					inline: true
				},
				{
					name: filters.data!.VNL.displayMode,
					value: filters.data!.VNL.icon,
					inline: true
				}
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu",
				iconURL: client.icon
			});

		return reply(interaction, { embeds: [embed] });
	}
};
