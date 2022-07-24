export function timeString(time: number) {
	const date = new Date(time * 1000);
	const hours = date.getUTCHours();
	const minutes = date.getUTCMinutes();
	const seconds = date.getUTCSeconds();
	const millies = date.getUTCMilliseconds();

	return `${hours > 0 ? hours.toString().padStart(2, "0") + ":" : ""}${minutes
		.toString()
		.padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${millies
		.toString()
		.padStart(3, "0")}`;
}
