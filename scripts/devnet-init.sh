#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$repo_root/devnet"
image="cometbft/cometbft:v0.38.26"
validator_count=4
full_node_count=1

lock="${out}.lock"

while ! mkdir "$lock" 2>/dev/null; do
  while [ -d "$lock" ] && [ ! -f "$out/node0/config/genesis.json" ]; do
    sleep 0.2
  done

  if [ -f "$out/node0/config/genesis.json" ]; then
    exit 0
  fi
done

scratch="${out}.tmp.$$"
trap 'rm -rf "$scratch"; rmdir "$lock" 2>/dev/null || true' EXIT

if [ -f "$out/node0/config/genesis.json" ]; then
  exit 0
fi

rm -rf "$out"

docker run --rm -v "$scratch:/devnet" "$image" testnet \
  --v "$validator_count" \
  --n "$full_node_count" \
  --o /devnet \
  --populate-persistent-peers \
  --hostname-prefix cometbft \
  --p2p-port 26656

for ((i = 0; i < validator_count + full_node_count; i++)); do
  abci_port=$((26658 + i * 10))
  config="$scratch/node$i/config/config.toml"

  sed -i.bak -E \
    -e "s#^proxy_app = .*#proxy_app = \"tcp://host.docker.internal:${abci_port}\"#" \
    -e 's#laddr = "tcp://127.0.0.1:26657"#laddr = "tcp://0.0.0.0:26657"#' \
    "$config"
  rm -f "$config.bak"

  genesis="$scratch/node$i/config/genesis.json"
  sed -i.bak -E 's/"chain_id": ".*"/"chain_id": "anchor-devnet"/' "$genesis"
  rm -f "$genesis.bak"
done

mv "$scratch" "$out"

echo "Initialized ${validator_count}-validator, ${full_node_count}-full-node devnet at ${out}" >&2
