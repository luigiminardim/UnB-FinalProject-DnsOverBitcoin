#!/bin/sh

# start bitcoind
bitcoind -daemon -datadir=/root/.bitcoin
IS_RUNNING=
while [ -z "$IS_RUNNING" ]; do
    sleep 1
    IS_RUNNING=$(bitcoin-cli getblockchaininfo)
done

# create minner wallet if it doesn't exist
WALLET_EXISTS=$(bitcoin-cli loadwallet "minner" | grep minner)
if [ -z "$WALLET_EXISTS" ]; then
    bitcoin-cli createwallet "minner"
fi


function mine_block {
    ADDRESS=$(bitcoin-cli -rpcwallet=minner getnewaddress "" bech32)
    echo "Generated block to address: $ADDRESS"
    bitcoin-cli -rpcwallet=minner generatetoaddress 1 "$ADDRESS"
}

# generate the first 101 blocks to the minner wallet
# this is needed to get the wallet to be able to spend the coins
CURRENT_BLOCK=$(bitcoin-cli getblockcount)
for i in $(seq $CURRENT_BLOCK 100); do
    mine_block
done

# generate blocks to the minner wallet
while true; do
    sleep 600 # 10 minutes
    mine_block
done