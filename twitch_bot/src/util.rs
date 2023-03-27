pub fn fmt_time(time: f64) -> String {
	let seconds = time as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((time - (time as u32) as f64) * 1000.0) as u16;

	let mut s = format!("{minutes:02}:{seconds:02}.{millis:03}");

	if hours > 0 {
		s = format!("{hours:02}:{s}");
	}

	s
}
