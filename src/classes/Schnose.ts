import {
	ApplicationCommand,
	Client,
	ClientOptions,
	Collection,
	EmbedBuilder,
	REST,
	Routes
} from "discord.js";
import { promises } from "fs";
import mongoose from "mongoose";

interface CustomCommand extends ApplicationCommand {
	execute: Function;
}

class SchnoseBot extends Client {
	public commands: Collection<string, CustomCommand> = new Collection();
	public icon: string;

	constructor(opts: ClientOptions) {
		super(opts);

		this.eventHandler();
		this.commandHandler();
		this.connectToDatabase(process.env.MONGODB);

		this.icon =
			process.env.ICON ||
			"https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png";
	}

	public run(token?: string) {
		if (!token) throw new SyntaxError("No Discord API Token was provided.");

		this.login(token)
			.then(() => console.log("The bot has been started."))
			.catch((err) => console.error(err));
	}

	private connectToDatabase(token?: string) {
		if (!token) throw new SyntaxError("No MongoDB token was provided.");

		mongoose
			.connect(token)
			.then(() => console.log("Successfully established connection to the database."))
			.catch((e: unknown) => console.error(e));
	}

	private async eventHandler() {
		const eventList: string[] = [];
		const eventFiles = await promises.readdir(`${process.cwd()}/dist/events`);

		eventFiles.forEach((file) => {
			let eventFile = require(`${process.cwd()}/dist/events/${file}`)?.default;
			if (!eventFile.name || !eventFile.execute)
				throw new SyntaxError("Incorrect event file structure");

			eventList.push(eventFile.name);
			this.on(eventFile.name, (...args) => eventFile.execute(...args, this));
		});

		console.log(`Successfully registered events:\n> ${eventList.join("\n> ")}`);
	}

	private async commandHandler() {
		const { DJS_TOKEN, BOT_ID, DEV_GUILD, MODE } = process.env;
		const rest = new REST({ version: "10" }).setToken(DJS_TOKEN!);

		const commands: JSON[] = [];
		const commandList: string[] = [];
		const commandFiles = await promises.readdir(`${process.cwd()}/dist/commands`);

		commandFiles.forEach((file) => {
			let commandFile = require(`${process.cwd()}/dist/commands/${file}`)?.default;
			if (!commandFile.data) throw new SyntaxError("incorrect command file structure");

			commands.push(commandFile.data.toJSON());
			commandList.push(commandFile.data.name);
			this.commands.set(commandFile.data.name, commandFile);
		});

		if (!BOT_ID) throw new SyntaxError("No `BOT_ID` has been specified.");
		if (!DEV_GUILD) throw new SyntaxError("No `DEV_SERVER` has been specified.");

		switch (MODE) {
			case "DEV":
				await rest.put(Routes.applicationGuildCommands(BOT_ID, DEV_GUILD), {
					body: commands
				});
				break;

			case "PROD":
				await rest.put(Routes.applicationGuildCommands(BOT_ID, DEV_GUILD), {
					body: commands
				});
				break;

			default:
				throw new SyntaxError("Please correctly specify your `MODE` environment variable.");
		}

		console.log(
			`${
				MODE === "DEV" ? "[LOCAL]" : "[GLOBAL]"
			} Successfully registered commands:\n> ${commandList.join("\n> ")}`
		);

		this.on("interactionCreate", async (interaction) => {
			if (interaction.isCommand()) {
				const command = this.commands.get(interaction.commandName) as CustomCommand;
				if (!command) return;

				await command.execute(interaction, this);
			} else if (interaction.isSelectMenu()) {
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
						.setFooter({ text: "(͡ ͡° ͜ つ ͡͡°)7", iconURL: this.icon });

					interaction.update({ embeds: [helpEmbed] });
				}
			}
		});
	}
}

export default SchnoseBot;
