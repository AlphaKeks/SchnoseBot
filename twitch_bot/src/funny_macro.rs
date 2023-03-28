use gokz_rs::{MapIdentifier, Mode, PlayerIdentifier};

macro_rules! parse_args {
    ( $message:expr, $( $t:ty ),+ ) => ({
		let message = $message;
		let mut used = Vec::new();
		(|| -> Result<_, &str> {
			Ok((
				$(
				message.split(' ').enumerate().find_map(|(idx, word)| {
					if used.contains(&idx) {
						return None;
					}

					if let Ok(parsed) = word.parse::<$t>() {
						used.push(idx);
						return Some(parsed);
					}

					None
				}).ok_or("")?
				),+
			))
		})()
	});
}

#[test]
fn basic() {
	let message = String::from("lionharder alphakeks skz");

	let (map, player, mode) = parse_args!(message, MapIdentifier, PlayerIdentifier, Mode).unwrap();
	assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
	assert_eq!(player, PlayerIdentifier::Name(String::from("alphakeks")));
	assert_eq!(mode, Mode::SimpleKZ);
}
