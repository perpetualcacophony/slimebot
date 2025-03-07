#!/bin/sh

# create slimebot user
addgroup --system slimebot --gid $GID
adduser --system slimebot --ingroup slimebot

# copy slimebot.toml
cp /slimebot.toml /etc/slimebot/

# make config directory readable
chown -R slimebot:slimebot /etc/slimebot/

su -s /bin/sh slimebot -c 'slimebot start $@'