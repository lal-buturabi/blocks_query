
#!/bin/bash
#
export ABI_PATH="/Users/user/works/gits/one_and_only/blocks_one/src/abi.json"
export RPC_URL1="http://3.23.124.61:8545" # stopped.

export RPC_URL2="http://3.133.2.70:8545" # syncing..
export RPC_URL3="http://3.20.106.105:8545" # syncing..

export DATABASE_NAME="Nexa_Events_Data_4"
export COLLECTION_NAME="events_table"
export MONGODB_URI="mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.2.5"

./target/release/blocks_query
