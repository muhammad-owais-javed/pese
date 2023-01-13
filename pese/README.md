Privacy Extension for Search Engines (PESE)
-------------------------------------------

# Test

Test first without Enarx Keep:

```sh
cargo run
```

Build release:

```sh
cargo build --release --target=wasm32-wasi
```

Run release without TEE:

```sh
cargo run --release
```

Run inside Enarx Keep:

```sh
enarx run --backend=nil --wasmcfgfile Enarx.toml target/wasm32-wasi/release/pese.wasm
```

# Execute test query

Open `http://127.0.0.1:34455/` on your browser.

```sh
curl "http://127.0.0.1:34455/search/?q=zcash"
# Fetch the result then, some /result/123456789
```

# Benchmarking

```sh
python benchmark.py
```
