// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate chrono;

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate postgres;
#[macro_use]
extern crate postgres_derive;

mod api;
mod database;
mod routes;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    use super::postgres;
    use super::std::num;
    use super::std::io;
    use super::serde_json;
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        foreign_links {
            Io(io::Error);
            Json(serde_json::Error);
            Num(num::ParseIntError);
            Pg(postgres::error::Error);
        }
    }
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
pub use errors::*;

use api::*;
use routes::Context;

// Use this macro to auto-generate the main. You may want to
// set the `RUST_BACKTRACE` env variable to see a backtrace.
quick_main!(run);

fn run() -> Result<()> {
    use std::io;
    use std::io::BufRead;

    let stdin = io::stdin();
    let mut ctxt: Context = Context::new();
    for (no, line) in stdin.lock().lines().enumerate() {
        match main_step(&mut ctxt, &line?).chain_err(|| format!("input line {}", no + 1)) {
            Ok(e) => main_ok(e),
            Err(e) => main_err(&e),
        }
    }

    Ok(())
}

fn main_step(ctx: &mut Context, line: &str) -> Result<Response> {
    let request: Request = read_call(line)?;
    let response: Response = request.resolve(ctx)?;
    Ok(response)
}

fn main_ok(e: Response) -> () {
    let response_json = match e {
        Response::Ok(ResponseInfo::Empty) => {
            let ok_json = json!({ "status": "OK" });
            ok_json.to_string()
        }
        Response::Ok(serializable) => {
            let ok_json = json!({ "status": "OK", "data": serializable });
            ok_json.to_string()
        }
        Response::NotImplemented => {
            let not_implemented_json = json!({ "status": "NOT IMPLEMENTED" });
            not_implemented_json.to_string()
        }
    };
    println!("{}", response_json);
}

fn main_err(e: &Error) -> () {
    let error_json = json!({ "status": "ERROR" });
    println!("{}", error_json.to_string());
    debug_main_err(e);
}

#[cfg(debug_assertions)]
fn debug_main_err(e: &Error) -> () {
    use std::io::{stderr, Write};
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

#[cfg(not(debug_assertions))]
fn debug_main_err(_: &Error) -> () {}
