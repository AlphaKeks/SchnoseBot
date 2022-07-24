import { Client } from "discord.js";
import { eventHandler } from "./handlers/eventHandler";
import "dotenv/config";

const schnose = new Client({ intents: 34576 });
eventHandler(schnose);

async function main(bot: Client, token: string) {
	bot
		.login(token)
		.then(() => console.log("The bot has been started."))
		.catch((e: unknown) => console.error(e));
}

main(schnose, process.env.DJS_TOKEN!);
