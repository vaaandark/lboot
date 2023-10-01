#![no_main]
#![no_std]

extern crate alloc;

use lboot::{image::start_kernel_image, config::Config};
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
    let bt = st.boot_services();
    info!("Start booting...");
    let config = Config::load_from_file(bt, cstr16!(r"\efi\boot\lboot.toml")).unwrap();
    let enties = config.parse().unwrap();
    for entry in &enties {
        println!("{}", entry);
    }
    if let Some(entry) = enties.first() {
        start_kernel_image(image_handle, bt, entry).unwrap();
    }
    Status::SUCCESS
}
