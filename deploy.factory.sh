# deploy dao

cd dao-factory

sh build.sh

cd ..

near deploy \
    --wasmFile out/connecus-factory.wasm \
    --initFunction "new" \
    --initArgs '{"token_contract_id": "connecus.testnet"}' \
    --accountId factory.connecus.testnet