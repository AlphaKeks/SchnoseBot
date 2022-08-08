import { Interaction, UserMention } from "discord.js";
import { getPlayer } from "gokz.js";
import userSchema from "../schemas/user";

export async function validateTarget(
	interaction: Interaction,
	input: null | string | UserMention
): Promise<{
	success: boolean;
	data?: { type: "name" | "steamID" | "mention" | null; value?: string };
	error?: string;
}> {
	const res: {
		success: boolean;
		data?: { type: "name" | "steamID" | "mention" | null; value?: string };
		error?: string;
	} = {
		success: false,
		data: { type: "name" },
	};

	if (!input) {
		const userDB = await userSchema.find({ discordID: interaction.user.id });
		if (!userDB[0]?.steamID) res.data = { type: null };
		else
			res.data = {
				type: "steamID",
				value: userDB[0].steamID,
			};
	} else {
		if (input.startsWith("<@") && input.endsWith(">"))
			res.data = { type: "mention" };
		else if (/STEAM_[0-1]:[0-1]:[0-9]+/.test(input)) {
			res.data = { type: "steamID", value: input };
		} else res.data = { type: "name" };
	}

	switch (res.data.type) {
		case null:
			res.error =
				"You did not specify a target and you also haven't saved a steamID in the database. You can do that with `/setsteam`, so you don't have to specify a target everytime you use a command.";
			break;
		case "mention": {
			input = input!.slice(2, -1);
			if (input.startsWith("!")) input = input.slice(1);
			const userDB = await userSchema.find({ discordID: input });
			if (!userDB[0]?.steamID)
				res.error =
					"The user you mentioned did not register a steamID in the database.";
			else res.success = true;
			res.data = {
				type: "steamID",
				value: userDB[0].steamID,
			};
			break;
		}
		case "name": {
			const player = await getPlayer(input!);
			if (!player.success) res.error = player.error;
			else {
				res.success = true;
				res.data.value = player.data!.steam_id;
			}
			break;
		}
		case "steamID":
			res.success = true;
			break;
	}
	return res;
}

const modeMap = new Map();
modeMap.set("kz_timer", "KZT");
modeMap.set("kz_simple", "SKZ");
modeMap.set("kz_vanilla", "VNL");
modeMap.set("KZT", "kz_timer");
modeMap.set("SKZ", "kz_simple");
modeMap.set("VNL", "kz_vanilla");
export default modeMap;
