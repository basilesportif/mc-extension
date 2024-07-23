#!/bin/bash

forge build
# Extract the 'abi' field from the source JSON and write it to the target file
jq '.abi' out/Counter.sol/Counter.json > ../gamelord/abi/Counter.json
# cd ~/kinode-work/mc-extension/sol-contracts
jq '.abi' out/Counter.sol/Counter.json > ../mcclient/ui/abi/Counter.json
echo "ABI updated successfully."