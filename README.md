# SchnoseBot

Discord Bot written in Rust with the Serenity framework for CS:GO KZ Commands

## Dev setup

First clone the repo:

```sh
$ git clone https://github.com/AlphaKeks/SchnoseBot.git
```

Initialize environment variables:

```sh
$ cp .env.tx .env
```

```
# login credentials
DISCORD_TOKEN=<[Bot token](https://discord.com/developers/applications)>
MONGODB=<MongoDB connection string>
STEAM_API=<Steam WebAPI Key>

# variables
MODE=<"DEV" or "PROD">
DEV_GUILD=<GuildID of your test server>
BOT_ID=<UserID of your bot's discord account>
ICON=<URL for embed icons>
```

Build the bot:

```sh
$ cargo build --release
```

Start the bot:

```sh
$ ./target/release/schnose
```
