#!/bin/bash
export DIST="export/flathub/iplan" &&
rm -rf $DIST &&
mkdir $DIST &&
rsync -a --exclude-from='../.gitignore' --exclude='ir.imansalmani.IPlan.Devel.json' --exclude='ir.imansalmani.IPlan.json' --exclude='build-aux' ../ $DIST &&
cp ir.imansalmani.IPlan.json $DIST/ &&
cd $DIST &&
mkdir .cargo &&
touch .cargo/config &&
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > .cargo/config &&
git init &&
git add . &&
git commit -m "Build commit" &&
cd ../ &&
tar -zcvf iplan.tar.gz iplan &&
sha256sum iplan.tar.gz > checksum
