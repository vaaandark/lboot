#![no_main]
#![no_std]

extern crate alloc;
use lboot::config::Config;
use lboot_test_runner::shutdown;
use log::info;
use uefi::prelude::*;
use uefi::{
    entry,
    table::{Boot, SystemTable},
    Handle,
};
use uefi_services::println;

#[entry]
fn main(image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    info!("Testing configuration file parsing now...");
    let config =
        Config::load_from_file(st.boot_services(), cstr16!(r"\efi\boot\lboot.toml")).unwrap();
    config
        .parse()
        .unwrap()
        .iter()
        .for_each(|e| println!("Found entry: {}", e));
    st.boot_services().stall(3_000_000);
    shutdown(st);
}
