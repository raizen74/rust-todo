#!/usr/bin/env bash

# navigate to directory
SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd $SCRIPTPATH
cd ../..
cd frontend
npm install
npm run build
cd ../ingress
cargo clean # clean the build artifacts to embed the frontend build into the binary
cargo run
