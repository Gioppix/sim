#!/usr/bin/env bash
set -euo pipefail

generate_ts_types() {
  echo "Generating TypeScript types..."
  export TS_RS_EXPORT_DIR="./bindings_new"
  cargo test export_bindings
  rm -rf ./bindings
  mv ./bindings_new ./bindings
  echo "Done. Types written to ./bindings/all.ts"
}

"$@"
