import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import { validateTarget } from "../lib/functions/schnose";
import { getMaps, getPB, getPlace, validateMap } from "gokz.js";
import userSchema from "../lib/schemas/user";
import "dotenv/config";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("pb")
		.setDescription("Check a player's personal best on a map.")
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
		)
		.addStringOption((o) => o.setName("target").setDescription("Specify a target.")),

	async execute(interaction: ChatInputCommandInteraction) {
		await interaction.deferReply();

		const inputMap = interaction.options.getString("map")!;
		const inputMode = interaction.options.getString("mode") || null;
		const inputTarget = interaction.options.getString("target") || null;

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

		const targetValidation = await validateTarget(interaction, inputTarget);
		if (!targetValidation.success) return reply(interaction, { content: targetValidation.error });

		const req = await Promise.all([
			await getPB(targetValidation.data!.value!, mapValidation.data!.name, 0, mode, true),
			await getPB(targetValidation.data!.value!, mapValidation.data!.name, 0, mode, false)
		]);

		let tpPlace: any = null;
		let proPlace: any = null;
		if (req[0].success) tpPlace = await getPlace(req[0].data!);
		if (req[1].success) proPlace = await getPlace(req[1].data!);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`[PB] - ${req[0].data?.player_name || req[1].data?.player_name} on ${
					mapValidation.data!.name
				}`
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
					value: `${parseTime(req[0].data?.time || 0)} (${
						tpPlace?.success ? `#${tpPlace?.data}` : ""
					})`,
					inline: true
				},
				{
					name: "PRO",
					value: `${parseTime(req[1].data?.time || 0)} (${
						proPlace?.success ? `#${proPlace?.data}` : ""
					})`,
					inline: true
				}
			])
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7",
				iconURL: process.env.ICON
			});

		return reply(interaction, {
			embeds: [embed]
		});
	}
};
