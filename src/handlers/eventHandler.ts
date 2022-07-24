import { promisify } from "util";
import glob from "glob";
import { Client } from "discord.js";

export async function eventHandler(client: Client) {
	const PG = promisify(glob);
	const events: string[] = [];

	(await PG(`${process.cwd()}/dist/events/*.js`)).map(async (file: string) => {
		const event = require(file);

		if (event.once)
			client.once(event.name, (...args: any) => event.execute(...args, client));
		else
			client.on(event.name, (...args: any) => event.execute(...args, client));

		events.push(event.name);
	});

	console.log(`Successfully registered events:\n${events}`);
}
