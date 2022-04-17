use anyhow::Result;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wire::Input;

use crate::wire::Output;

mod wire;

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    let input = Input {
        name: "Rust".into(),
        num: 10,
    };
    let serialized_input = serde_json::to_string(&input)?;

    let stdin = ReadPipe::from(serialized_input);
    let stdout = WritePipe::new_in_memory();

    let wasi = WasiCtxBuilder::new()
        .stdin(Box::new(stdin.clone()))
        .stdout(Box::new(stdout.clone()))
        .build();

    let module = Module::from_file(&engine, "target/wasm32-wasi/debug/wasi-demo.wasm")?;

    let mut store = Store::new(&engine, wasi);

    linker
        .module(&mut store, "", &module)
        .expect("linking the function");
    linker
        .get_default(&mut store, "")
        .expect("should get the wasi runtime")
        .typed::<(), (), _>(&store)
        .expect("should type the function")
        .call(&mut store, ())
        .expect("should call the function");

    drop(store);

    let contents: Vec<u8> = stdout
        .try_into_inner()
        .map_err(|_err| anyhow::Error::msg("sole remaining reference"))?
        .into_inner();
    let output: Output = serde_json::from_slice(&contents)?;

    println!("output: {:?}", output);

    Ok(())
}
