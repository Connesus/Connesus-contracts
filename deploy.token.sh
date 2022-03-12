# deploy dao

near deploy \
    --wasmFile out/connecus-token.wasm \
    --initFunction "new" \
    --initArgs '{
        "metadata": {
            "spec": "ft-1.0.0",
            "name": "ManhnvCoin",
            "symbol": "MNC",
            "icon": null,
            "reference": null,
            "reference_hash": null,
            "decimals": 1
        },
        "owner_id": "manhndev.testnet",
        "total_supply": "1000000000000"
    }' \
    --accountId connecus-token.manhndev.testnet