#!/bin/bash

set -e

printf ">> Enter the dispute contract account ID: "
read DISPUTE_CONTRACT

printf ">> Enter the store factory contract account ID: "
read FACTORY_CONTRACT

printf "\n>> Building & deploying dispute contract"
./scripts/build.sh dispute
near deploy --wasmFile res/dispute.wasm --accountId $DISPUTE_CONTRACT --initFunction new --initArgs '{}'

printf "\n>> Adding dispute contract address to store contract then building it"
sed -i -e "s/ddd7.testnet/$DISPUTE_CONTRACT/g" store/src/lib.rs
./scripts/build.sh store
sed -i -e "s/$DISPUTE_CONTRACT/ddd7.testnet/g" store/src/lib.rs

printf "\n>> Building & deploying store factory contract"
./scripts/build.sh store-factory
near deploy --wasmFile res/store_factory.wasm --accountId $FACTORY_CONTRACT --initFunction new --initArgs '{}'

echo "


Successfully built and deployed the contracts 
to the following accounts:

----------------------------------------------
>> Dispute Contract: $DISPUTE_CONTRACT
>> Store Factory Contract: $FACTORY_CONTRACT
----------------------------------------------"