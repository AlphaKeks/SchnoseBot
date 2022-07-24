import { z } from "zod";

export const Player = z.object({
	steamid64: z.string(),
	steam_id: z.string(),
	is_banned: z.boolean(),
	total_records: z.number(),
	name: z.string(),
});
export type Player = z.infer<typeof Player>;
