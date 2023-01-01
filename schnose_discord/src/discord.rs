use crate::SchnoseError::{self, *};

#[derive(Debug, Clone, Copy)]
pub struct Mention(u64);

impl std::ops::Deref for Mention {
	type Target = u64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::str::FromStr for Mention {
	type Err = SchnoseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let Ok(regex) = regex::Regex::new(r#"^<@[0-9]+>$"#) else {
			return Err(Parsing(String::from("Mention")))
		};

		if !regex.is_match(s) {
			return Err(InvalidMention(s.to_owned()));
		}

		let user_id = s.replace("<@", "");
		let user_id = user_id.replace('>', "");
		let user_id = user_id.parse::<u64>().expect("This should be a valid u64.");

		Ok(Self(user_id))
	}
}
