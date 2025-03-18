#!/bin/sh

# start bitcoind
bitcoind -daemon -datadir=/root/.bitcoin
IS_RUNNING=
while [ -z "$IS_RUNNING" ]; do
    sleep 1
    IS_RUNNING=$(bitcoin-cli getblockchaininfo)
done

# create minner wallet if it doesn't exist
WALLET_EXISTS=$(bitcoin-cli listwallets | grep minner)
if [ -z "$WALLET_EXISTS" ]; then
    bitcoin-cli createwallet "minner"
fi

# generate blocks to the minner wallet
while true; do
    ADDRESS=$(bitcoin-cli -rpcwallet=minner getnewaddress "" bech32)
    echo "Generated block to address: $ADDRESS"
    bitcoin-cli -rpcwallet=minner generatetoaddress 1 "$ADDRESS"
    sleep 600 # 10 minutes
done