import { ActivityType, Client } from "discord.js";

module.exports = {
	name: "ready",
	once: true,

	execute(client: Client) {
		// discord randomly resets bot statuses, no idea why.
		setInterval(() => {
			client.user!.setActivity("with your balls", {
				type: ActivityType.Playing
			});
		}, 60000);
		console.log(`${client.user!.tag} is now online.`);
	}
};
