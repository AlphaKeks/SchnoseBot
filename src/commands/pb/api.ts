import { KZMap } from "src/lib/types/gokz";
import * as g from "../../lib/functions/gokz";

export async function apiCall(
	map: KZMap,
	mode: string,
	player: { success: true; data: { type: string; value: string } }
) {
	const [TP, PRO] = await Promise.all([
		g.getPB(map.name, mode, player, 0, true),
		g.getPB(map.name, mode, player, 0, false),
	]);

	return {
		TP: TP.error ? null : TP,
		PRO: PRO.error ? null : PRO,
	};
}
