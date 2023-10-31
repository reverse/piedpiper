use libpiedpiper::Encoder;
use std::path::Path;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let filename = Path::new("shakespeare.txt");

    let fs = filename.metadata().unwrap().len();
    println!("File is {} bytes", fs);

    let mut encoder = Encoder::open(filename)?;
    let encoded = encoder.encode()?;
    println!("Encoded value has {} bytes", encoded.len() / 8);

    let decoded = encoder.decode(encoded)?;
    println!("Decoded value is {}", decoded);

    Ok(())
}
