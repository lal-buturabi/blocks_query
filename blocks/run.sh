#!/bin/bash
#
export MONGODB_URI="mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.2.5"

cargo build 1>/dev/null

./target/debug/blocks
