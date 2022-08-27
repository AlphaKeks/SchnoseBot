import { SlashCommandBuilder, ChatInputCommandInteraction, EmbedBuilder } from "discord.js";
import { reply } from "../lib/functions/discord";
import { parseTime } from "../lib/functions/util";
import { validateTarget } from "../lib/functions/schnose";
import { getMapKZGO, getMaps, getPB, getPlace, validateMap } from "gokz.js";
import userSchema from "../lib/schemas/user";
import modeMap from "gokz.js/lib/api";
import SchnoseBot from "src/classes/Schnose";

export default {
	data: new SlashCommandBuilder()
		.setName("bpb")
		.setDescription("Check a player's personal best on a bonus.")
		.addStringOption((o) => o.setName("map").setDescription("Specify a map.").setRequired(true))
		.addIntegerOption((o) => o.setName("course").setDescription("Specify a bonus."))
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

	async execute(interaction: ChatInputCommandInteraction, client: SchnoseBot) {
		await interaction.deferReply();

		const inputMap = interaction.options.getString("map")!;
		const inputCourse = interaction.options.getInteger("course") || 0;
		const inputMode = interaction.options.getString("mode") || null;
		const inputTarget = interaction.options.getString("target") || null;

		const globalMaps = await getMaps();
		if (!globalMaps.success) return reply(interaction, { content: globalMaps.error });

		const mapValidation = await validateMap(inputMap, globalMaps.data!);
		if (!mapValidation.success) return reply(interaction, { content: mapValidation.error });

		const KZGOMap = await getMapKZGO(mapValidation.data!.name);
		if (!KZGOMap.success) return reply(interaction, { content: KZGOMap.error });

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
			await getPB(targetValidation.data!.value!, mapValidation.data!.name, inputCourse, mode, true),
			await getPB(targetValidation.data!.value!, mapValidation.data!.name, inputCourse, mode, false)
		]);

		let tpPlace: any = null; // eslint-disable-line
		let proPlace: any | null = null; // eslint-disable-line
		if (req[0].success) tpPlace = await getPlace(req[0].data!);
		if (req[1].success) proPlace = await getPlace(req[1].data!);

		const embed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(
				`[BPB ${inputCourse}] ${
					req[0].data?.player_name || req[1].data?.player_name
						? `${req[0].data?.player_name || req[1].data?.player_name} on ${
								mapValidation.data!.name
						  }` // eslint-disable-line
						: `${mapValidation.data!.name}`
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
					value: `${req[0].data?.time ? parseTime(req[0].data.time) : "😔"} ${
						tpPlace && tpPlace.success ? `(#${tpPlace?.data})` : `${req[0].success ? "?" : ""}`
					}`,
					inline: true
				},
				{
					name: "PRO",
					value: `${req[1].data?.time ? parseTime(req[1].data.time) : "😔"} ${
						proPlace && proPlace.success ? `(#${proPlace?.data})` : `${req[1].success ? "?" : ""}`
					}`,
					inline: true
				}
			])
			.setFooter({
				text: `(͡ ͡° ͜ つ ͡͡°)7 | ${modeMap.get(mode)}`,
				iconURL: client.icon
			});

		return reply(interaction, {
			embeds: [embed]
		});
	}
};
