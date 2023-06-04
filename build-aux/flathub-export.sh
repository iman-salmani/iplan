#!/bin/bash
set -e

DIST="./export/flathub/iplan"
FLATHUB="$(dirname $DIST)"
rm -rf $DIST
mkdir -p $DIST
rsync -a --exclude-from='../.gitignore' --exclude='ir.imansalmani.IPlan.Devel.json' --exclude='ir.imansalmani.IPlan.json' --exclude='build-aux' ../ $DIST
cp ir.imansalmani.IPlan.json $DIST/
mkdir $DIST/.cargo
cargo vendor "$DIST"/vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > $DIST/.cargo/config.toml
tar -zcf "$FLATHUB"/iplan.tar.gz $DIST
sha256sum "$FLATHUB"/iplan.tar.gz > "$FLATHUB"/checksum
