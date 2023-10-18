extern crate cbindgen;

use std::env;

// TODO parse and optimize yaml files into serde binary for faster
// startup time.

fn main() {

    // let mut build_dir = get_cargo_target_dir().unwrap();
    // build_dir.push("regexes");

    // let _ = std::fs::create_dir(build_dir);

    // let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/regexes/bots.yml"));
    // let value: serde_yaml::Value = serde_yaml::from_str(contents).unwrap();

    build_cpp_header();

}

fn build_cpp_header() {
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
        }
    }
}

// we can use this to maybe compress the yaml files into a more compact representation
// before placing into the binary.
#[allow(dead_code)]
fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let profile = std::env::var("PROFILE")?;
    let mut target_dir = None;
    let mut sub_path = out_dir.as_path();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.ok_or("not found")?;
    Ok(target_dir.to_path_buf())
}
