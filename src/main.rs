use std::error::Error;

use wire::{Input, Output};

mod wire;

#[link(wasm_import_module = "host")]
extern "C" {
    fn get_input_size() -> i32;
    fn get_input(ptr: i32);
    fn set_output(ptr: i32, size: i32);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mem_size = unsafe { get_input_size() };

    let mut buf: Vec<u8> = Vec::with_capacity(mem_size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(ptr);

    let input_buf = unsafe {
        get_input(ptr as i32);
        Vec::from_raw_parts(ptr, mem_size as usize, mem_size as usize)
    };

    println!("input_buf = {:?}", input_buf);

    let input: Input = serde_json::from_slice(&input_buf).map_err(|e| {
        eprintln!("ser: {e}");
        e
    })?;

    println!("input = {:?}", input);

    let names: Vec<String> = (0..input.num).map(|_idx| input.name.clone()).collect();

    let output = Output { names };
    let serialized = serde_json::to_vec(&output).map_err(|e| {
        eprintln!("de: {e}");
        e
    })?;
    let size = serialized.len() as i32;
    let ptr = serialized.as_ptr();
    std::mem::forget(ptr);

    unsafe {
        set_output(ptr as i32, size);
    }

    Ok(())
}
