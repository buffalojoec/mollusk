solana program dump MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr ./src/elf/memo.so -u mainnet-beta
solana program dump Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo ./src/elf/memo-v1.so -u mainnet-beta
solana slot -u mainnet-beta | xargs -I {} sed -i '' 's|// Last updated at mainnet-beta slot height: .*|// Last updated at mainnet-beta slot height: {}|' ./src/lib.rs