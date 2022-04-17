# Sharing data between hosts with stdio

This repository has an accompanying [blog post](https://petermalmgren.com/serverside-wasm-data/).

## Running

### 1. Build the WASM

```bash
$ cargo build --target wasm32-wasi
```

### 2. Run the example

```bash
$ cargo run --example wasi
```