use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    println!("{:#?}", "starting");
    let region = Region::new(GLOBAL);
    {
        let mut vec = Vec::with_capacity(10);
        println!("1 {:#?}", region.change());
        vec.push(1);
        vec.push(2);
        vec.push(3);
        println!("2 {:#?}", region.change());

        vec.push(1);
        println!("3 {:#?}", region.change());

        vec.shrink_to_fit();
        println!("4 {:#?}", region.change());
    }
    println!("5 {:#?}", region.change());
}
