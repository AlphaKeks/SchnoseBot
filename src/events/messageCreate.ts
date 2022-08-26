import { Message } from "discord.js";
import SchnoseBot from "../classes/Schnose";

export default {
	name: "messageCreate",

	execute(msg: Message, client: SchnoseBot) {
		if (msg.author.id === client.user!.id) return;

		if (msg.content.toLowerCase().includes("bing?")) {
			if (msg.author.id === "291585142164815873") return msg.reply({ content: "chilling 🍦" });
			else
				return msg.reply({
					content: `${Math.round(Math.random()) ? "chilling 🍦" : "no 😔"}`
				});
		}
	}
};
