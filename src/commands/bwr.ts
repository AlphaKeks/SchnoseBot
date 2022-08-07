import {
	SlashCommandBuilder,
	ChatInputCommandInteraction,
	EmbedBuilder,
} from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import {
	getMapKZGO,
	getMaps,
	getWR,
	validateCourse,
	validateMap,
} from "gokz.js";
import userSchema from "../lib/schemas/user";
import "dotenv/config";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("bwr")
		.setDescription("Check the World Record on a bonus.")
		.addStringOption((o) =>
			o.setName("mapname").setDescription("Specify a map.").setRequired(true)
		)
		.addNumberOption((o) =>
			o.setName("course").setDescription("Specify a bonus.")
		)
		.addStringOption((o) =>
			o.setName("mode").setDescription("Specify a mode.").setChoices(
				{
					name: "KZT",
					value: "kz_timer",
				},
				{
					name: "SKZ",
					value: "kz_simple",
				},
				{
					name: "VNL",
					value: "kz_vanilla",
				}
			)
		),

	async execute(interaction: ChatInputCommandInteraction) {
		interaction.deferReply();

		const inputMap = interaction.options.getString("mapname")!;
		const inputCourse = interaction.options.getNumber("course") || 0;
		const inputMode = interaction.options.getString("mode") || null;

		const globalMaps = await getMaps();
		if (!globalMaps.success)
			return reply(interaction, { content: globalMaps.error });

		const mapValidation = await validateMap(inputMap, globalMaps.data);
		if (!mapValidation.success)
			return reply(interaction, { content: mapValidation.error });

		const KZGOMap = await getMapKZGO(mapValidation.data.name);
		if (!KZGOMap.success) return reply(interaction, { content: KZGOMap.error });

		const courseValidation = await validateCourse(KZGOMap.data, inputCourse);
		if (!courseValidation)
			return reply(interaction, { content: "Please specify a valid course." });

		let mode: string;
		if (inputMode) mode = inputMode;
		else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]?.mode)
				return reply(interaction, {
					content:
						"You must either specify a mode or set a default value using `/mode`.",
				});
			else mode = userDB[0].mode;
		}

		const req = await Promise.all([
			await getWR(mapValidation.data.name, inputCourse, mode, true),
			await getWR(mapValidation.data.name, inputCourse, mode, false),
		]);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`[BWR ${inputCourse}] - ${mapValidation.data.name}`)
			.setURL(
				`https://kzgo.eu/maps/${mapValidation.data.name}&bonus=${inputCourse}`
			)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${mapValidation.data.name}.jpg`
			)
			.addFields([
				{
					name: "TP",
					value: `${parseTime(req[0].data?.time || 0)} (${
						req[0].data?.player_name || "-"
					})`,
					inline: true,
				},
				{
					name: "PRO",
					value: `${parseTime(req[1].data?.time || 0)} (${
						req[1].data?.player_name || "-"
					})`,
					inline: true,
				},
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: process.env.ICON,
			});

		return reply(interaction, {
			embeds: [embed],
		});
	},
};
