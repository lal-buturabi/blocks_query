#!/bin/bash
#
export ABI_PATH="/data/works/gits/one_and_only/blocks_one/src/abi.json"
export RPC_URL1="http://3.23.124.61:8545" # stopped.

export RPC_URL2="http://3.133.2.70:8545" # syncing..
export RPC_URL3="http://3.20.106.105:8545" # syncing..

./target/release/blocks_one
