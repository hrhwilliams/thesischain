#! /bin/sh

diesel migration run

export CONTRACT_ADDRESS=$(cat /shared/contract_address)
echo "Using contract address: $CONTRACT_ADDRESS"

exec $@