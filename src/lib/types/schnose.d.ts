import { Player } from "gokz.js/lib/types";

interface PlayerProfile extends Player {
	mode?: string;
	tpPoints?: number;
	proPoints?: number;
	tpFinishes?: number[];
	tpPerc?: number;
	proFinishes?: number[];
	proPerc?: number;
	tpRecords?: numbers;
	proRecords?: number;
	rank?: string;
}
