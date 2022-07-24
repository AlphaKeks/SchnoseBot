import { KZMap } from "src/lib/types/gokz";
import * as g from "../../lib/functions/gokz";

export async function apiCall(map: KZMap, mode: string, course: number) {
	const [TP, PRO] = await Promise.all([
		g.getWR(map.name, mode, course, true),
		g.getWR(map.name, mode, course, false),
	]);

	return {
		TP: TP.error ? null : TP,
		PRO: PRO.error ? null : PRO,
	};
}
