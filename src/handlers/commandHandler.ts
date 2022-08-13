import { Client, Collection, EmbedBuilder, Interaction, Routes } from "discord.js";
import { REST } from "@discordjs/rest";
import { promisify } from "util";
import glob from "glob";
import "dotenv/config";

export async function commandHandler(client: Client) {
	const PG = promisify(glob);
	const commands: JSON[] = [];
	const commandList: string[] = [];
	const commandFiles = await PG(`${process.cwd()}/dist/commands/*.js`);
	client.commands = new Collection();

	for (const command of commandFiles) {
		let commandFile = require(command);
		if (commandFile.default) commandFile = commandFile.default;
		if (process.env.MODE !== "DEV" && process.env.MODE !== "PROD")
			return console.log("Please configure your .env properly. [1]");

		commands.push(commandFile.data.toJSON());
		commandList.push(commandFile.data.name);
		client.commands.set(commandFile.data.name, commandFile);
	}

	// registering commands
	client.once("ready", async () => {
		const rest = new REST({ version: "9" }).setToken(process.env.DJS_TOKEN!);

		switch (process.env.MODE) {
			case "DEV":
				await rest.put(Routes.applicationGuildCommands(client.user!.id, process.env.DEV_GUILD!), {
					body: {}
				});
				await rest.put(Routes.applicationGuildCommands(client.user!.id, process.env.DEV_GUILD!), {
					body: commands
				});
				console.log(`[LOCAL] Successfully registered commands:\n${commandList}`);
				break;
			case "PROD":
				await rest.put(Routes.applicationCommands(client.user!.id), {
					body: commands
				});
				console.log(`[GLOBAL] Successfully registered commands:\n${commandList}`);
				break;
			default:
				return console.error("Please configure your .env properly. [2]");
		}
	});

	// handling commands
	client.on("interactionCreate", async (interaction: Interaction) => {
		if (interaction.isChatInputCommand()) {
			const command = client.commands.get(interaction.commandName);
			if (!command) return;

			await command.execute(interaction).catch((e: unknown) => console.error(e));
		}

		if (interaction.isSelectMenu()) {
			if (interaction.customId === "commands-menu") {
				let embedTitle = "";
				let embedDescription = "";

				switch (interaction.values[0]) {
					case "tutorial-value":
						embedTitle = `Getting Started`;
						embedDescription = `You can use this bot simply by typing a \`/\` and previewing all the available commands. Many of them will require you to specify a \`target\` or \`mode\` (e.g. when using \`/pb\`). Most often you probably want to check your own stats and you probably also mainly play 1 specific mode. Because it's annoying to specify the same parameters over and over again, this bot uses a database to store those values for you. By using \`/setsteam\` you can save your steamID in the bot's database so it remembers it for every command you're gonna use in the future. Note that, if you specify a \`target\` on any command, it will prioritize that over your database entries. You can also set your preferred mode with \`/mode\`. Priority here is the same as with \`/setsteam\`.\n\nYou can also @mention other people when using commands! If you want to check your best friend's PB on some map you can simply @ them and, if they have set their steamID using \`/setsteam\`, the bot will use it.\n\nIf you have any suggestions or find bugs with the bot you can either message \`AlphaKeks#9826\` on Discord or open an Issue on [GitHub](https://github.com/AlphaKeks/SchnoseBot/issues).`;
						break;
					case "apistatus-value":
						embedTitle = `/apistatus`;
						embedDescription = `This command will tell you whether the GlobalAPI is up or not.`;
						break;
					case "bpb-value":
						embedTitle = `/bpb`;
						embedDescription = `This command will show you your (or another player's) best time on a bonus course.\nYou can specify the following parameters:\n> map*\n> course\n> mode\n> target\n\n*required`;
						break;
					case "bwr-value":
						embedTitle = `/bwr`;
						embedDescription = `This command will show you the WR of a given bonus course.\nYou can specify the following parameters:\n> map*\n> course\n> mode\n\n*required`;
						break;
					case "db-value":
						embedTitle = `/db`;
						embedDescription = `This command will show you your current database entries.\nExample output:\n> userID: 291585142164815873\n> steamID: STEAM_1:1:161178172\n> mode: kz_simple`;
						break;
					case "invite-value":
						embedTitle = `/invite`;
						embedDescription = `This command will give you a link to invite the bot to your own Discord Server.`;
						break;
					case "map-value":
						embedTitle = `/map`;
						embedDescription = `This command will give you detailed information on a map, such as it's name, tier, mappers, etc.`;
						break;
					case "mode-value":
						embedTitle = `/mode`;
						embedDescription = `This command will either show you your current mode preference or you can specify a mode and overwrite your previous preference. This will allow you to use a lot of other commands without needing to specify a mode everytime.`;
						break;
					case "nocrouch-value":
						embedTitle = `/nocrouch`;
						embedDescription = `If you LongJump without crouching at the end, you will lose a lot of distance; typically around 11 units. This command will give you a close approximation of how far your jump could have been if you had crouched. The command assumes that your jump was done on 128t and that your \`max\` was the speed you had at the end of your jump.`;
						break;
					case "pb-value":
						embedTitle = `/pb`;
						embedDescription = `This command will show you your (or another player's) best time on a map.\nYou can specify the following parameters:\n> map*\n> mode\n> target\n\n*required`;
						break;
					case "random-value":
						embedTitle = `/random`;
						embedDescription = `This command will give you a random KZ map to play. You can sort by tiers as well, if you want to.`;
						break;
					case "recent-value":
						embedTitle = `/recent`;
						embedDescription = `This command will show you your (or another player's) most recent PB.`;
						break;
					case "setsteam-value":
						embedTitle = `/setsteam`;
						embedDescription = `This command will store your steamID in Schnose's database so that it can be used in other commands to check player specific information.`;
						break;
					case "unfinished-value":
						embedTitle = `/unfinished`;
						embedDescription = `This command will give you a list of maps you have not yet completed.\nYou can specify the following parameters:\n> tier\n> mode\n> runtype\n> target`;
						break;
					case "wr-value":
						embedTitle = `/wr`;
						embedDescription = `This command will show you the World Record of a given map.\nYou can specify the following parameters:\n> map*\n> mode\n\n*required`;
						break;
				}

				const helpEmbed = new EmbedBuilder()
					.setColor([116, 128, 194])
					.setTitle(embedTitle)
					.setDescription(embedDescription)
					.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: process.env.ICON });

				interaction.update({ embeds: [helpEmbed] });
			}
		}
	});
}
