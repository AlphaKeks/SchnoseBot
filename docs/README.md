# SchnoseBot

SchnoseBot is a Discord Bot for [CS:GO KZ](https://forum.gokz.org/) that leverages the
[official KZ GlobalAPI](https://kztimerglobal.com/swagger/index.html?urls.primaryName=V2),
[n4vyn's](https://github.com/n4vyn) [KZ:GO API](https://kzgo.eu/) and my own
[SchnoseAPI](https://github.com/AlphaKeks/SchnoseAPI) for fetching information about players, maps,
records, etc.

It is supposed to give you the ability to use commands available in-game in Discord! This includes
commands such as `/pb`, `/wr`, `/maptop` and many more! See [the
Wiki](https://github.com/AlphaKeks/SchnoseAPI/wiki) for more examples as well as screenshots. Since
many commands require a `mode` or `player` argument (e.g. `/pb`), the bot also uses a MySQL database
to store preference options for each user so you don't have to specify those options over and over
again (see `/setsteam` and `/mode`). If you want to see all available commands in Discord use
`/help`.

You can invite the bot via [this link](https://bot.schnose.xyz/).

# Setup

If you want to run your own instance of the bot, you can follow these steps:

1. Install [rustup](https://rustup.rs/). If you are on Linux or MacOS you will get an installation script to run in your shell. If you are on Windows you will download a `.exe`.
2. Clone this repo

```sh
git clone https://github.com/AlphaKeks/SchnoseBot.git
```

3. Copy the `config.toml.example` to some location of your choice and fill out all the values.
4. Run `cargo build` or `cargo build --release` (for an optimized build).
5. Start the bot with `cargo run` (or `cargo run --release`).
	- If you copy the `config.toml.example` to your current directory and name it `config.toml`, the bot will automatically pick it up. Otherwise you can tell it the location by passing the `--config` flag like so: `cargo run -- --config /path/to/config`

You will need a MySQL database with a table of the following definition:

```sql
CREATE TABLE users (
	name       VARCHAR(255)      NOT NULL,
	discord_id BIGINT   UNSIGNED NOT NULL PRIMARY KEY,
	steam_id   VARCHAR(255),
	mode       SMALLINT UNSIGNED
);
```
