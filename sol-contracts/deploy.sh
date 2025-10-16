#!/bin/bash

if [ -f ../.env ]; then
  source ../.env
else
  echo ".env file not found!"
  exit 1
fi

forge script
case $VITE_CURRENT_CHAIN_ID in
    1)
        echo "Deploying to Mainnet"
        forge script --rpc-url $VITE_MAINNET_RPC_URL script/Deploy.s.sol --broadcast --account mainnet
        ;;
    11155111)
        echo "Deploying to Sepolia"
        forge script --rpc-url $VITE_SEPOLIA_RPC_URL script/Deploy.s.sol --broadcast --account sepolia
        ;;
    10)
        echo "Deploying to Optimism"
        forge script --rpc-url $VITE_OPTIMISM_RPC_URL script/Deploy.s.sol --broadcast --account optimism
        ;;
    31337)
        echo "Deploying to Anvil"
        forge script --rpc-url $VITE_ANVIL_RPC_URL script/Deploy.s.sol --broadcast --account anvil
        ;;
    *)
        echo "Unknown chain ID: $VITE_CURRENT_CHAIN_ID"
        exit 1
        ;;
esac

echo "deploy successful"
