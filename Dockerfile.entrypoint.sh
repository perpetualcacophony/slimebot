#!/bin/sh

# create slimebot user
addgroup --system slimebot --gid $GID
adduser --system slimebot --ingroup slimebot

# create secrets directory
mkdir /etc/slimebot
mkdir /etc/slimebot/secrets

# copy docker secrets to new secrets dir
cp /run/secrets/* /etc/slimebot/secrets/

# make secrets readable
chmod -R o+r /etc/slimebot/secrets

# run app as slimebot user
su -s /bin/sh slimebot -c slimebot