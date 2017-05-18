// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
use errors::*;

// Use this macro to auto-generate the main. You may want to
// set the `RUST_BACKTRACE` env variable to see a backtrace.
quick_main!(run);

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<()> {
    use std::fs::File;

    // This operation will fail
    File::open("tretrete")
        .chain_err(|| "unable to open tretrete file")?;

    Ok(())
}
