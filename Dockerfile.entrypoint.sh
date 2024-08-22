#!/bin/sh

addgroup --system slimebot --gid $GID
adduser --system slimebot --ingroup slimebot
id