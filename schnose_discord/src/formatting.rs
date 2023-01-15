/// Turns an amount of seconds into a nicely formatted string:
///
/// ```
/// let time: f32 = 1263.7832;
/// assert_eq!("00:21:03.78", &format_time(time));
/// ```
pub(crate) fn format_time(secs_float: f64) -> String {
	let seconds = secs_float as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((secs_float - (secs_float as u32) as f64) * 1000.0) as u16;

	let mut s = format!("{:02}:{:02}.{:03}", minutes, seconds, millis);

	if hours > 0 {
		s = format!("{:02}:{}", hours, s);
	}

	s
}

pub fn map_link(map_name: &str) -> String {
	format!("https://kzgo.eu/maps/{}", map_name)
}

pub fn map_thumbnail(map_name: &str) -> String {
	format!(
		"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
		map_name
	)
}
