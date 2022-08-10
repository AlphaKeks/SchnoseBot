import { Client, Collection, Interaction, Routes } from "discord.js";
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
	});
}
