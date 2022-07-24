import { KZMap } from "src/lib/types/gokz";
import * as g from "../../lib/functions/gokz";

export async function apiCall(map: KZMap, mode: string) {
	const [TP, PRO] = await Promise.all([
		g.getWR(map.name, mode, 0, true),
		g.getWR(map.name, mode, 0, false),
	]);

	return {
		TP: TP.error ? null : TP,
		PRO: PRO.error ? null : PRO,
	};
}
