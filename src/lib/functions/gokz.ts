import axios from "axios";
import { Player } from "../types/gokz";

export async function playerSteamID(steamID: string) {
	const player: Player = await axios
		.get(
			`https://kztimerglobal.com/api/v2.0/players/steamid/${encodeURIComponent(
				steamID
			)}`
		)
		.then((response) => {
			return response.data[0];
		})
		.catch((e: unknown) => console.error(e));

	if (Player.safeParse(player).success) return { success: true, data: player };
	else return { success: false, error: "Invalid Input." };
}
