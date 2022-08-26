import { ActivityType, Client } from "discord.js";

export default {
	name: "ready",

	execute(client: Client) {
		// discord randomly resets bot statuses, no idea why.
		setInterval(() => {
			client.user!.setActivity("kz_epiphany_v2", {
				type: ActivityType.Playing
			});
		}, 60000);
		console.log(`${client.user!.tag} is now online.`);
	}
};
