#!/bin/sh
mkdir -p nucleoid/source/usr/lib/
git clone https://gitlab.com/nucleoid/nucleoid.git nucleoid/source/usr/lib/nucleoid/
npm install --prefix nucleoid/source/usr/lib/nucleoid/
rm -Rf nucleoid/source/usr/lib/nucleoid/.git

mkdir -p nucleoid/debian
mv nucleoid/source/usr/lib/nucleoid/scripts/debian/* nucleoid/debian/

mkdir -p nucleoid/source/var/lib/nucleoid/init/
mkdir -p nucleoid/source/opt/nucleoid/

mkdir -p nucleoid/source/etc/nucleoid/
echo "{}" > nucleoid/source/etc/nucleoid/configuration.json
