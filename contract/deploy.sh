#!/bin/sh
set -e

# 12sec ethereum timings, 0.15 gwei / gas
anvil --host 0.0.0.0 --block-time 12 --block-base-fee-per-gas 150000000 &
ANVIL_PID=$!

until cast block-number --rpc-url http://localhost:8545 >/dev/null 2>&1; do
  sleep 0.5
done

forge script script/KeyDirectory.sol:KeyDirectoryScript \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast

# get address
# cp broadcast/KeyDirectory.sol/31337/run-latest.json /shared/run-latest.json
grep -m 1 "contractAddress" broadcast/KeyDirectory.sol/31337/run-latest.json | awk -F '"' '{print $4}' > /shared/contract_address

wait $ANVIL_PID
