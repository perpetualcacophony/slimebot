#!/bin/sh

addgroup --system slimebot --gid $GID
adduser --system slimebot --ingroup slimebot
su -s /bin/sh slimebot -c slimebot