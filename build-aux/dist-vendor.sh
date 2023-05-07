#!/bin/bash
export DIST="export/flathub"
rsync -a --exclude-from='.gitignore' ./ $DIST
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > $DIST/.cargo/config