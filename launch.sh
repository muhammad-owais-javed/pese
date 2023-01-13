#!/bin/bash
set -e
# Select the backend. SEV and SGX are TEEs.
backend="nil" # Options are sev, sgx, kvm or nil

if netstat -an | grep 34455 | grep -q LISTEN
then
   echo "Port 34455 is not available";
   exit 1
else
   echo "Port 34455 is available";
fi

cd pese

# Build release wasm
cargo build --release --target=wasm32-wasi

# Run inside Enarx keep
enarx run --backend=$backend --wasmcfgfile Enarx.toml target/wasm32-wasi/release/pese.wasm &
PID=$!

echo "Running Enarx Keep: "$PID
echo "Quit with keys Ctrl+C"

( trap exit SIGINT ; read -r -d '' _ </dev/tty ) # Wait for Ctrl-C
kill $PID
