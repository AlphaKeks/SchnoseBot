import { Message } from "discord.js";
import "dotenv/config";

module.exports = {
	name: "messageCreate",

	execute(msg: Message) {
		if (msg.author.id === process.env.BOT_ID) return;

		if (msg.content.toLowerCase().includes("bing?")) {
			if (msg.author.id === "291585142164815873") return msg.reply({ content: "chilling 🍦" });
			else
				return msg.reply({
					content: `${Math.round(Math.random()) ? "chilling 🍦" : "no 😔"}`
				});
		}
	}
};
