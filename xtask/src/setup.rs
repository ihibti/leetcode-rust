use std::io;

pub fn run() -> Result<(), io::Error> {
    println!("Note: 'cargo setup' has been replaced by './lc setup'.");
    println!("Run ./lc setup from the repo root instead.");
    println!("(./lc works even before Rust/Cargo are installed.)");
    Ok(())
}
