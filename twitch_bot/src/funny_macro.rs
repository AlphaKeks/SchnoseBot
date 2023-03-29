#![allow(clippy::needless_question_mark)]

use {
	color_eyre::Result,
	gokz_rs::{MapIdentifier, Mode, PlayerIdentifier},
};

trait FromStrWrapper {
	type Output;
	type Err;
	fn from_opt_str(s: &str) -> Result<Self::Output, Self::Err>;
}

impl<T: std::str::FromStr> FromStrWrapper for Option<T> {
	type Output = Option<T>;
	type Err = Option<Self::Output>;
	fn from_opt_str(s: &str) -> Result<Self::Output, Self::Err> {
		Ok(s.parse::<T>().ok())
	}
}

macro_rules! from_str_opt {
	($t:ty) => {
		impl FromStrWrapper for $t {
			type Output = $t;
			type Err = <$t as std::str::FromStr>::Err;
			fn from_opt_str(s: &str) -> Result<Self::Output, Self::Err> {
				s.parse()
			}
		}
	};
}

from_str_opt!(Mode);
from_str_opt!(PlayerIdentifier);
from_str_opt!(MapIdentifier);

trait GenParseError {
	fn missing() -> crate::Error;
	fn incorrect() -> crate::Error;
}

macro_rules! gen_parse_err {
	($t:ty, $missing:expr, $incorrect:expr) => {
		impl GenParseError for $t {
			fn missing() -> crate::Error {
				$missing
			}

			fn incorrect() -> crate::Error {
				$incorrect
			}
		}

		impl GenParseError for Option<$t> {
			fn missing() -> crate::Error {
				$missing
			}

			fn incorrect() -> crate::Error {
				$incorrect
			}
		}
	};
}

gen_parse_err!(
	Mode,
	crate::Error::MissingArgs { missing: String::from("mode") },
	crate::Error::IncorrectArgs { expected: String::from("valid mode") }
);
gen_parse_err!(
	PlayerIdentifier,
	crate::Error::MissingArgs { missing: String::from("player") },
	crate::Error::IncorrectArgs { expected: String::from("valid player") }
);
gen_parse_err!(
	MapIdentifier,
	crate::Error::MissingArgs { missing: String::from("map") },
	crate::Error::IncorrectArgs { expected: String::from("valid map") }
);

macro_rules! parse_args {
    ( $message:expr, $( $t:ty ),+ ) => ({
		(|| -> Result<_, crate::Error> {
			let mut message: Vec<&str> = $message.split(' ').collect();
			Ok(($({
				let mut idx = None;
				for (i, word) in message.iter().enumerate() {
					if idx.is_some() {
						break;
					}
					if <$t as FromStrWrapper>::from_opt_str(word).is_ok() {
						idx = Some(i);
					}
				}
				message.push("");

				let idx = idx.ok_or(<$t as GenParseError>::missing())?;
				let word = message.remove(idx);
				let parsed = <$t as FromStrWrapper>::from_opt_str(word).expect("we found it earlier");
				Result::<_, crate::Error>::Ok(parsed)
			}?),+))
		})()
	});
}

#[test]
fn map_only() -> Result<()> {
	let message = "lionharder";

	let map = parse_args!(message, MapIdentifier)?;
	assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));

	Ok(())
}

#[test]
fn map_and_mode() -> Result<()> {
	let message = "lionharder skz";

	let (map, mode) = parse_args!(message, MapIdentifier, Mode)?;
	assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
	assert_eq!(mode, Mode::SimpleKZ);

	Ok(())
}

#[test]
fn map_and_mode_and_player() -> Result<()> {
	let message = "lionharder skz alphakeks";

	let (map, mode, player) = parse_args!(message, MapIdentifier, Mode, PlayerIdentifier)?;
	assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
	assert_eq!(mode, Mode::SimpleKZ);
	assert_eq!(player, PlayerIdentifier::Name(String::from("alphakeks")));

	let message = "lionharder alphakeks skz";

	let (map, mode, player) = parse_args!(message, MapIdentifier, Mode, PlayerIdentifier)?;
	assert_eq!(map, MapIdentifier::Name(String::from("lionharder")));
	assert_eq!(mode, Mode::SimpleKZ);
	assert_eq!(player, PlayerIdentifier::Name(String::from("alphakeks")));

	Ok(())
}
