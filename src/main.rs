// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

mod api;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    use super::serde_json;
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            Json(serde_json::Error);
        }
    }
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
pub use errors::*;

// Use this macro to auto-generate the main. You may want to
// set the `RUST_BACKTRACE` env variable to see a backtrace.
quick_main!(run);

fn run() -> Result<()> {
    use std::io::Write;
    use std::io::stderr;

    let mut line = String::new();
    loop {
        match main_step(&mut line) {
            Ok(e) => println!("{:?}", e),
            Err(e) => {
                println!("JSON ERROR");

                /// following lines should be only in debug mode
                let stderr = &mut stderr();
                let errmsg = "Error writing to stderr";

                writeln!(stderr, "error: {}", e).expect(errmsg);
                for e in e.iter().skip(1) {
                    writeln!(stderr, "caused by: {}", e).expect(errmsg);
                }
                // The backtrace is not always generated. Try to run this example
                // with `RUST_BACKTRACE=1`.
                if let Some(backtrace) = e.backtrace() {
                    writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
                }
            }
        }
    }
}

fn main_step(mut line: &mut String) -> Result<()> {
    use std::io;
    use api::*;

    io::stdin().read_line(&mut line)?;
    let _: Api = read_call(&line)?;

    Ok(())
}
