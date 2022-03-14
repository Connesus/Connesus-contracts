# deploy dao

near deploy \
    --wasmFile out/connecus-token.wasm \
    --initFunction "new" \
    --initArgs '{
        "metadata": {
            "spec": "ft-1.0.0",
            "name": "Connecus",
            "symbol": "CEUS",
            "icon": "https://bafybeibplttj6muri65lq7vr6f55k3gdpsmet66l5gfycyuzmtubxy66te.ipfs.dweb.link/hearts.png",
            "reference": null,
            "reference_hash": null,
            "decimals": 1
        },
        "owner_id": "manhndev.testnet",
        "total_supply": "1000000000000"
    }' \
    --accountId connecus.testnet