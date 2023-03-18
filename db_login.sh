#!/bin/sh
mariadb -h 192.168.178.160 -D schnose -u schnose -p`grep "url" discord_bot/config.toml | sed 's/.*".*schnose:\(.*\)@.*"/\1/'`
