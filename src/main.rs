use std::{error::Error, io::stdin};

use wire::{Input, Output};

mod wire;

fn main() -> Result<(), Box<dyn Error>> {
    let input: Input = serde_json::from_reader(stdin()).map_err(|e| {
        eprintln!("ser: {e}");
        e
    })?;

    let names: Vec<String> = (0..input.num).map(|_idx| input.name.clone()).collect();

    let output = Output { names };
    let serialized = serde_json::to_string(&output).map_err(|e| {
        eprintln!("de: {e}");
        e
    })?;

    println!("{serialized}");

    Ok(())
}
