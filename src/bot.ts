import { Client } from "discord.js";
import { eventHandler } from "./handlers/eventHandler";
import { commandHandler } from "./handlers/commandHandler";
import mongoose from "mongoose";
import "dotenv/config";

const schnose = new Client({ intents: 34576 });
eventHandler(schnose);
commandHandler(schnose);

async function main(bot: Client, token: string) {
	bot
		.login(token)
		.then(() => console.log("The bot has been started."))
		.catch((e: unknown) => console.error(e));

	if (!process.env.MONGODB) console.log("No database found.");
	mongoose
		.connect(process.env.MONGODB!)
		.then(() => console.log("Successfully established connection to the database."))
		.catch((e: unknown) => console.error(e));
}

main(schnose, process.env.DJS_TOKEN!);
