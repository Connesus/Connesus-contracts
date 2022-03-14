# deploy dao

cd connesus-dao

sh build.sh

cd ..

near deploy \
    --wasmFile out/connecus-factory.wasm \
    --initFunction "migrate" \
    --initArgs '{"token_contract_id": "connecus.testnet"}' \
    --accountId factory.connecus.testnet