# deploy dao

cd connesus-dao

sh build.sh

cd ..

near deploy \
    --wasmFile out/connecus-dao.wasm \
    --initFunction "new" \
    --initArgs '{
        "metadata": {
            "name": "test dao",
            "purpose": "String",
            "thumbnail": "String",
            "symbol": "String",
            "facebook": null,
            "youtube": null,
            "twitter": null,
            "discord": null,
            "instagram": null
        },
        "token_contract_id": "connecus-token.manhndev.testnet"
    }' \
    --accountId connecus-dao.manhndev.testnet