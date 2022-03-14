# deploy dao

cd connesus-dao

sh build.sh

cd ..

near deploy \
    --wasmFile out/connecus-dao.wasm \
    --initFunction "migrate" \
    --initArgs '{"owner_id": "manhndev.testnet"}' \
    --accountId connecus-dao.manhndev.testnet