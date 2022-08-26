import { ActivityType } from "discord.js";
import SchnoseBot from "src/classes/Schnose";

export default {
	name: "ready",

	execute(client: SchnoseBot) {
		console.log(`${client.user!.tag} is now online.`);

		// discord randomly resets bot statuses, no idea why.
		setInterval(() => {
			client.user!.setActivity("kz_epiphany_v2", {
				type: ActivityType.Playing
			});
		}, 60000);
	}
};
