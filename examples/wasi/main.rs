use std::sync::{Arc, Mutex};

use anyhow::Result;
use wasi_common::WasiCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wire::Input;

use crate::wire::Output;

mod wire;

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    let wasi = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build();

    let module = Module::from_file(&engine, "target/wasm32-wasi/debug/wasi-demo.wasm")?;

    let mut store = Store::new(&engine, wasi);
    let memory_ty = MemoryType::new(1, None);
    Memory::new(&mut store, memory_ty)?;

    let input = Input {
        name: "hey".into(),
        num: 5,
    };
    let buf = serde_json::to_vec(&input).expect("should serialize");
    let mem_size: i32 = buf.len() as i32;

    linker
        .func_wrap("host", "get_input_size", move || -> i32 { mem_size })
        .expect("should define the function");
    linker
        .func_wrap(
            "host",
            "get_input",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return Err(Trap::new("failed to find host memory")),
                };
                let offset = ptr as u32 as usize;
                match mem.write(&mut caller, offset, &buf) {
                    Ok(_) => {}
                    _ => return Err(Trap::new("failed to write to host memory")),
                };
                Ok(())
            },
        )
        .expect("should define the function");

    let output: Arc<Mutex<Output>> = Arc::new(Mutex::new(Output { names: vec![] }));
    let output_ = output.clone();
    linker
        .func_wrap(
            "host",
            "set_output",
            move |mut caller: Caller<'_, WasiCtx>, ptr: i32, capacity: i32| {
                let output = output_.clone();
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return Err(Trap::new("failed to find host memory")),
                };
                let offset = ptr as u32 as usize;
                let mut buffer: Vec<u8> = vec![0; capacity as usize];
                match mem.read(&caller, offset, &mut buffer) {
                    Ok(()) => {
                        println!(
                            "Buffer = {:?}, ptr = {}, capacity = {}",
                            buffer, ptr, capacity
                        );
                        match serde_json::from_slice::<Output>(&buffer) {
                            Ok(serialized_output) => {
                                let mut output = output.lock().unwrap();
                                *output = serialized_output;
                                Ok(())
                            }
                            Err(err) => {
                                let msg = format!("failed to serialize host memory: {}", err);
                                Err(Trap::new(msg))
                            }
                        }
                    }
                    _ => Err(Trap::new("failed to read host memory")),
                }
            },
        )
        .expect("should define the function");

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

    let output = output.lock();
    println!("output: {:?}", output);

    Ok(())
}
