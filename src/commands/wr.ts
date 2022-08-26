import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import { getMaps, getWR, validateMap } from "gokz.js";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";

export default {
	data: new SlashCommandBuilder()
		.setName("wr")
		.setDescription("Check the World Record on a map.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true))
		.addStringOption((o) =>
			o.setName("mode").setDescription("Specify a mode.").setChoices(
				{
					name: "KZT",
					value: "kz_timer"
				},
				{
					name: "SKZ",
					value: "kz_simple"
				},
				{
					name: "VNL",
					value: "kz_vanilla"
				}
			)
		),

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputMap = interaction.options.getString("map")!;
		const inputMode = interaction.options.getString("mode") || null;

		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const mapValidation = await validateMap(inputMap, globalMaps.data!);
		if (!mapValidation.success) return reply(interaction, { content: mapValidation.error });

		let mode: string;
		if (inputMode) mode = inputMode;
		else {
			const userDB = await userSchema.find({ discordID: interaction.user.id });
			if (!userDB[0]?.mode)
				return reply(interaction, {
					content: "You must either specify a mode or set a default value using `/mode`."
				});
			else mode = userDB[0].mode;
		}

		const req = await Promise.all([
			await getWR(mapValidation.data!.name, 0, mode, true),
			await getWR(mapValidation.data!.name, 0, mode, false)
		]);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`[WR] - ${mapValidation.data!.name} (${modeMap.get(
					req[0].data?.mode || req[1].data?.mode
				)})`
			)
			.setURL(`https://kzgo.eu/maps/${mapValidation.data!.name}`)
			.setThumbnail(
				`https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/${
					mapValidation.data!.name
				}.jpg`
			)
			.addFields([
				{
					name: "TP",
					value: `${parseTime(req[0].data?.time || 0)} (${req[0].data?.player_name || "-"})`,
					inline: true
				},
				{
					name: "PRO",
					value: `${parseTime(req[1].data?.time || 0)} (${req[1].data?.player_name || "-"})`,
					inline: true
				}
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: client.icon
			});

		return reply(interaction, {
			embeds: [embed]
		});
	}
};
