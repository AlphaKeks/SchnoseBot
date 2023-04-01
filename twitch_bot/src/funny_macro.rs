macro_rules! parse_args {
    ( $message:expr => $opt:literal $t:ty ) => ({
		(|| -> std::result::Result<Option<$t>, $crate::Error> {
			for (i, word) in $message.iter().enumerate() {
				if let Ok(parsed) = dbg!(word).parse::<$t>() {
					let _ = $message.remove(i);
					return Ok(Some(parsed));
				}
			}
			Ok(None)
		})()
	});

    ( $message:expr => $t:ty ) => ({
		(|| -> std::result::Result<_, $crate::Error> {
			for (i, word) in $message.iter().enumerate() {
				if let Ok(parsed) = dbg!(word).parse::<$t>() {
					let _ = $message.remove(i);
					return Ok(parsed);
				}
			}
			Err(<$t as $crate::error::GenParseError>::incorrect())
		})()
    });

    ( $message:expr, $( $($opt:literal)? $t:ty ),+ ) => ({
		(|| -> std::result::Result<_, $crate::Error> {
			let mut message: Vec<&str> = $message.split(' ').collect();
			Ok((
				$({
					dbg!(&message);
					parse_args!(message => $($opt)? $t)?
				}),+
			))
		})()
    });
}

pub(crate) use parse_args;

#[cfg(test)]
mod tests {
	use {
		super::parse_args,
		color_eyre::Result,
		gokz_rs::{MapIdentifier, Mode, PlayerIdentifier},
	};

	#[test]
	fn map_only() -> Result<()> {
		let message = "lionharder";

		let map = parse_args!(message, MapIdentifier)?;
		assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));

		let message = "";

		let map = parse_args!(message, MapIdentifier);
		assert!(map.is_err());

		let message = "";

		let map = parse_args!(message, "opt" MapIdentifier)?;
		assert!(map.is_none());

		let message = "lionharder";

		let map = parse_args!(message, "opt" MapIdentifier)?;
		assert!(map.is_some());
		assert_eq!(map, Some(MapIdentifier::Name(String::from("lionharder"))));

		Ok(())
	}

	#[test]
	fn map_and_mode() -> Result<()> {
		let message = "lionharder skz";

		let (map, mode) = parse_args!(message, MapIdentifier, Mode)?;
		assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
		assert_eq!(mode, Mode::SimpleKZ);

		let message = "";

		let parse_result = parse_args!(message, MapIdentifier, Mode);
		assert!(parse_result.is_err());

		let message = "";

		let (map, mode) = parse_args!(message, "opt" MapIdentifier, "opt" Mode)?;
		assert!(map.is_none());
		assert!(mode.is_none());

		let message = "lionharder";

		let (map, mode) = parse_args!(message, "opt" MapIdentifier, "opt" Mode)?;
		assert!(map.is_some());
		assert!(mode.is_none());

		let message = "skz";

		let (mode, map) = parse_args!(message, "opt" Mode, "opt" MapIdentifier)?;
		assert!(mode.is_some());
		assert!(map.is_none());

		Ok(())
	}

	#[test]
	fn map_maybe_mode_maybe_player() -> Result<()> {
		let message = "lionharder skz alphakeks";

		let (map, mode, player) = parse_args!(message, MapIdentifier, Mode, PlayerIdentifier)?;
		assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
		assert_eq!(mode, Mode::SimpleKZ);
		assert_eq!(player, PlayerIdentifier::Name(String::from("alphakeks")));

		let message = "lionharder skz";

		let (map, mode, player) =
			parse_args!(message, MapIdentifier, "opt" Mode, "opt" PlayerIdentifier)?;
		assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
		assert_eq!(mode, Some(Mode::SimpleKZ));
		assert_eq!(player, None);

		let message = "lionharder alphakeks";

		let (map, mode, player) =
			parse_args!(message, MapIdentifier, "opt" Mode, PlayerIdentifier)?;
		assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
		assert_eq!(mode, None);
		assert_eq!(player, PlayerIdentifier::Name(String::from("alphakeks")));

		Ok(())
	}
}
