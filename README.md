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
```

Initialize environment variables:

```sh
$ cp .env.tx .env
```

```
# login credentials
DJS_TOKEN=somecooltokenigotfromdiscord.com/developers
MONGODB=amongodbconnectionstring

# variables
MODE=DEV
DEV_GUILD=theserveridofmytestserver
BOT_ID=thebotsid
ICON=aurltoacoolicon
```

Build the bot:

```sh
$ npm run build
```

Start the bot:

```sh
$ npm run start
```
