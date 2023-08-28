// use tracking_allocator::AllocationRegistry;
// use tracking_allocator::Allocator;

// #[global_allocator]
// static GLOBAL: Allocator<System> = Allocator::system();

// use std::env;
use std::process::ExitCode;

use clap::Parser;

use rust_device_detector::device_detector::parse;
use rust_device_detector::http::server;

#[derive(Parser, Debug)]
/// A commandline user agent detection tool
///
/// This is a long explanation
#[command(version)]
struct Args {
    /// Run in interactive mode.
    ///
    /// In interactive mode, each stdin line will be parsed
    /// as a user agent, and we will return on stout, one single
    /// line of json as a result.
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Run as an http server.
    ///
    /// This will run the command as an http server, listening on the
    /// specified port or 8080 by default. You submit one line of user
    /// agent and you will get back a response in json.
    #[arg(short = 's', long = "server")]
    server: bool,

    /// Port to run on, when in http server mode.
    #[arg(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// When in cli mode (the default) this is the user agent to parse.
    ///
    /// Always remember escape shell arguments!
    #[arg(required_unless_present_any(["interactive", "server"]))]
    useragent: Option<String>,
}

// use std::alloc::System;
// use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

// #[global_allocator]
// static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[tokio::main]
async fn main() -> Result<(), ExitCode> {
    //  let stdout_tracker = rust_device_detector::tracking::StdoutTracker::new();
    //  AllocationRegistry::set_global_tracker(stdout_tracker)
    //      .expect("no other global tracker should be set yet");
    //  tracking_allocator::AllocationRegistry::enable_tracking();

    //    sc_core::setup::binary_setup();

    // let reg = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);

    let args = Args::parse();

    if args.interactive {
        let mut ua = String::with_capacity(50); // may also use with_capacity if you can guess
        while std::io::stdin().read_line(&mut ua).unwrap() > 0 {
            let headers = None;
            let detection = parse(&ua, headers).unwrap();
            println!("{}", detection.to_value());

            ua.clear(); // clear to reuse the buffer
        }
    } else if args.server {
        eprintln!("server mode");

        // tokio::spawn(async move {
        // let reg: Region<'static, System> = Region::new(&GLOBAL);
        server(args.port).await;
        // }).await;
    } else {
        match args.useragent {
            None => {
                eprintln!("No user agent specified");
                return Ok(());
            }

            Some(ua) => {
                // eprintln!("ua: {}", ua);
                let headers = None;
                let detection = parse(&ua, headers).unwrap();

                // TODO json
                println!("{}", detection.to_value());
            }
        }
    }

    // let ch = reg.change();
    // println!("allocations over entire run: {:#?} remaining {}", ch, ch.bytes_allocated - ch.bytes_deallocated);

    Ok(())
}
