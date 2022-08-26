# SchnoseBot

discord.js bot for CS:GO KZ commands

## Dev setup

First clone the repo:

```sh
$ git clone https://github.com/AlphaKeks/SchnoseBot.git
```

Install all the dependencies:

```sh
$ npm i
$ npm i -D
```

Initialize environment variables:

```sh
$ cp .env.tx .env
```

```
# login credentials
DJS_TOKEN=<[Bot token](https://discord.com/developers/applications)>
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
$ npm run build
```

Start the bot:

```sh
$ npm run start
```
