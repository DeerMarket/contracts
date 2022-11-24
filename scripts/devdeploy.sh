#!/bin/bash

spinner()
{
    local pid=$!
    local delay=0.1
    local spinstr='|/-\'
    while [ "$(ps a | awk '{print $1}' | grep $pid)" ]; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

set -e

printf ">> Cleaning up old accounts"
rm -rf neardev && mkdir -p neardev/dispute && mkdir -p neardev/store_factory

printf "\n>> Building & deploying dispute contract"
{
./scripts/build.sh dispute
near dev-deploy --wasmFile res/dispute.wasm --projectKeyDirectory neardev/dispute  --initFunction new --initArgs '{}'
DISPUTE_CONTRACT=$(<neardev/dispute/dev-account)
} & spinner

printf "\n>> Adding dispute contract address to store contract then building it"
{
sed -i -e "s/ddd7.testnet/$DISPUTE_CONTRACT/g" store/src/lib.rs
./scripts/build.sh store
sed -i -e "s/$DISPUTE_CONTRACT/ddd7.testnet/g" store/src/lib.rs
} &> /dev/null & spinner

printf "\n>> Building & deploying store factory contract"
{
./scripts/build.sh store-factory
near dev-deploy --wasmFile res/store_factory.wasm --projectKeyDirectory neardev/store_factory --initFunction new --initArgs '{}'
FACTORY_CONTRACT=$(<neardev/store_factory/dev-account)
} &> /dev/null & spinner

echo "


Successfully built and deployed the contracts 
to new dev accounts with the following addresses:

----------------------------------------------
>> Dispute Contract: $DISPUTE_CONTRACT
>> Store Factory Contract: $FACTORY_CONTRACT
----------------------------------------------"