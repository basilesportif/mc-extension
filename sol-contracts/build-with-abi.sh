#!/bin/bash

forge build
# Extract the 'abi' field from the source JSON and write it to the target file
jq '.abi' out/Counter.sol/Counter.json > ../gamelord/gamelord/abi/Counter.json
jq '.abi' out/Counter.sol/Counter.json > ../mcclient/ui/src/abi/Counter.json
echo "ABI updated successfully."