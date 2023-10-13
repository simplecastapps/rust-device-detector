use anyhow::Result;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use once_cell::sync::Lazy;

use rust_device_detector::device_detector::DeviceDetector;

pub(crate) static DD: Lazy<DeviceDetector> = Lazy::new(|| DeviceDetector::new());

// use stats_alloc::{Stats, INSTRUMENTED_SYSTEM};
// pub fn memory_test(f: &dyn Fn() -> Result<()>) -> Result<Stats> {
//     let reg = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);
//     f()?;
//
//     // difference in memory before and after function runs.
//     let ch = reg.change();
//
//     Ok(ch)
// }

pub fn file_paths(path: &str) -> Result<Vec<PathBuf>> {
    let files = glob::glob(path)
        .expect("text fixtures")
        .map(|x| x.expect("glob"))
        .collect::<Vec<_>>();

    Ok(files)
}

pub fn files(path: &str) -> Result<Vec<BufReader<File>>> {
    let files = file_paths(path)?
        .into_iter()
        .map(|x| BufReader::new(File::open(x).expect("file")))
        .collect::<Vec<_>>();

    Ok(files)
}
