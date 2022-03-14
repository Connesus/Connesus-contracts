# deploy dao

cd connesus-dao

sh build.sh

cd ..

near deploy \
    --wasmFile out/connecus-dao.wasm \
    --initFunction "new" \
    --initArgs '{
        "metadata": {
            "name": "CZ Binance",
            "purpose": "Build the future with blockchain",
            "thumbnail": "https://pbs.twimg.com/profile_images/1470780411747844096/vpxt_095_400x400.jpg",
            "symbol": "CZB",
            "facebook": "facebook.com",
            "youtube": "youtube.com",
            "twitter": "twitter.com",
            "discord": "discord.com",
            "instagram": "instagram.com"
        },
        "token_contract_id": "connecus-token.manhndev.testnet",
        "owner_id": "manhndev.testnet"
    }' \
    --accountId connecus.testnet