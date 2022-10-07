use std::fmt::Write;

pub fn timestring(secs_float: f32) -> String {
	let seconds = secs_float as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((secs_float - (secs_float as u32) as f32) * 1000.0) as u16;

	println!("{}", &millis);

	let mut s = String::new();

	let _ = write!(&mut s, "{:02}:{:02}.{:03}", minutes, seconds, millis);

	if hours > 0 {
		s = format!("{:02}:{}", hours, s);
	}

	s
}
