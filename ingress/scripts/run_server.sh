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
TO_DO_DB_URL='postgresql://username:mysecretpassword@localhost:5432/to_do' cargo run
