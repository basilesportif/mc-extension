#!/bin/bash

forge build
# Extract the 'abi' field from the source JSON and write it to the target file
cp out/Gamelord.sol/Gamelord.json ../gamelord/gamelord/abi/Gamelord.json
cp out/Gamelord.sol/Gamelord.json ../mcclient/ui/src/abi/Gamelord.json
echo "ABI updated successfully."
