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
DISCORD_TOKEN=<Discord API token>
MONGO_URL=<MongoDB connection string>
STEAM_API=<Steam WebAPI Key>

# variables
RUST_LOG=schnose=<log level> # e.g. `WARN`
MODE=DEV # `DEV` or `PROD`
DEV_GUILD=<Discord GuilID>
ICON_URL=https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png
```

Build the bot:

```sh
$ cargo build --release
```

Start the bot:

```sh
$ ./target/release/schnose
```
