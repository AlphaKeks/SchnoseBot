import {
	SlashCommandBuilder,
	ChatInputCommandInteraction,
	EmbedBuilder,
	ActionRowBuilder,
	SelectMenuBuilder,
	APIMessageActionRowComponent
} from "discord.js";
import { reply } from "../lib/functions/discord";
import "dotenv/config";

module.exports = {
	data: new SlashCommandBuilder()
		.setName("help")
		.setDescription("Get a list of all usable commands."),

	async execute(interaction: ChatInputCommandInteraction) {
		const helpEmbed = new EmbedBuilder()
			.setColor("#7480C2")
			.setTitle("Help Menu")
			.setDescription(
				"**(͡ ͡° ͜ つ ͡͡°)/**\n\nUse the menu below to get information on any command.\nIf you find any bugs or have any ideas for new features / improving existing ones, you can message `AlphaKeks#9826`.\n\nGitHub Page: https://github.com/AlphaKeks/SchnoseBot\nSteam Group: https://steamcommunity.com/groups/schnose"
			)
			.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: process.env.ICON });

		const commandMenu = new ActionRowBuilder().addComponents(
			new SelectMenuBuilder()
				.setCustomId("commands-menu")
				.setPlaceholder("Please select a category.")
				.addOptions(
					{
						label: `Getting Started`,
						description: `Quick start guide`,
						value: "tutorial-value"
					},
					{
						label: `/apistatus`,
						description: `Check the GlobalAPI Status.`,
						value: "apistatus-value"
					},
					{
						label: `/bpb`,
						description: `Check someone's Personal Best on a bonus of a map.`,
						value: `bpb-value`
					},
					{
						label: `/bwr`,
						description: `Check a Bonus World Record on a map.`,
						value: `bwr-value`
					},
					{
						label: `/db`,
						description: `Check your current Database entries.`,
						value: `db-value`
					},
					{
						label: `/invite`,
						description: `Invite Schnose to your server!`,
						value: `invite-value`
					},
					{
						label: `/map`,
						description: `Get detailed information on a map.`,
						value: `map-value`
					},
					{
						label: `/mode`,
						description: `Save your preferred gamemode in Schnose's database.`,
						value: `mode-value`
					},
					{
						label: `/nocrouch`,
						description: `Approximate potential distance of a nocrouch jump.`,
						value: `nocrouch-value`
					},
					{
						label: `/pb`,
						description: `Check someone's personal best on a map.`,
						value: `pb-value`
					},
					{
						label: `/random`,
						description: `Get a random KZ map. You can sort by filters if you want :)`,
						value: `random-value`
					},
					{
						label: `/recent`,
						description: `Get a player's most recent Personal Best.`,
						value: `recent-value`
					},
					{
						label: `/setsteam`,
						description: `Save your steamID in Schnose's database.`,
						value: `setsteam-value`
					},
					{
						label: `/unfinished`,
						description: `Check which maps you still have to complete.`,
						value: `unfinished-value`
					},
					{
						label: `/wr`,
						description: `Check the World Record of a map.`,
						value: `wr-value`
					}
				)
		);

		//@ts-ignore idk why discord.js has to be this complicated but it works :tf:
		reply(interaction, { embeds: [helpEmbed], components: [commandMenu], ephemeral: true });
	}
};
