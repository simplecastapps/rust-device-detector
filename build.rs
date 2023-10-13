extern crate cbindgen;

use std::env;

// TODO parse and optimize yaml files into serde binary for faster
// startup time.

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let res = cbindgen::Builder::new().with_crate(crate_dir).generate();

    match res {
        Ok(res) => {
            res.write_to_file("includes/rdd.h");
        }
        Err(e) => {
            // for some reason it can't just show you the error. you have to get rustc to print it
            // for you which it will tell you how to do on the last line, but if you missed it...
            // > rustc -Z parse-only /home/toad/working/rust-device-detector/src/ffi.rs
            // not sure why it doesn't just run that itself and show you the error.
            eprintln!("Error generating bindings: {}", e);

            std::process::exit(1);
        }
    }
}
