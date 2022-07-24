import { SlashCommandBuilder } from "@discordjs/builders";
import axios from "axios";
import { EmbedBuilder } from "discord.js";
import { CommandInteraction } from "discord.js";
import { reply } from "../../lib/functions/discord";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("apistatus")
		.setDescription("Check the GlobalAPI Status."),

	async execute(interaction: CommandInteraction) {
		async function checkStatus() {
			let status: any = [];
			await axios
				.get("https://status.global-api.com/api/v2/summary.json")
				.then((r) => {
					const apiStatus = {
						status: r.data.status.description,
						frontend: r.data.components[0].status,
						backend: r.data.components[1].status,
					};
					status.push(apiStatus);
				})
				.catch((e: unknown) => {
					console.error(e);
					status = undefined;
				});
			await axios
				.get("https://status.global-api.com/api/v2/incidents/unresolved.json")
				.then((r) => {
					const apiIncidents = {
						latest: r.data.incidents[0]?.name || null,
					};
					status.push(apiIncidents);
				})
				.catch((e: unknown) => {
					console.error(e);
					status = undefined;
				});
			return status;
		}

		const apiStatus = await checkStatus();
		if (!apiStatus) return reply(interaction, { content: "API Error." });

		const description = apiStatus[1].latest
			? `**Latest Incident**\n${apiStatus[1].latest}\n`
			: "no recent incidents";

		const statusEmbed = new EmbedBuilder()
			.setColor([116, 128, 194])
			.setTitle(`${apiStatus[0]!.status}`)
			.setThumbnail(
				"https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png"
			)
			.setDescription(description)
			.addFields(
				{
					name: "FrontEnd",
					value: `${apiStatus[0]!.frontend}`,
					inline: true,
				},
				{
					name: "BackEnd",
					value: `${apiStatus[0]!.backend}`,
					inline: true,
				}
			)
			.setFooter({
				text: "(͡ ͡° ͜ つ ͡͡°)7 | schnose.eu/church",
				iconURL: process.env.ICON,
			});
		return reply(interaction, { embeds: [statusEmbed] });
	},
};
