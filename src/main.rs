#![no_main]
#![no_std]

extern crate alloc;

use lboot::{image::start_kernel_image, config::Config, menu::select_in_menu};
use log::info;
use uefi::prelude::*;
use uefi::{
    entry,
    table::{Boot, SystemTable},
    Handle,
};

#[entry]
fn main(image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    info!("Start booting...");
    let config = Config::load_from_file(st.boot_services(), cstr16!(r"\efi\boot\lboot.toml")).unwrap();
    let entries = config.parse().unwrap();
    let entry = select_in_menu(&mut st, &entries);
    info!("Loading {}", entry);
    start_kernel_image(image_handle, st.boot_services(), entry).unwrap();
    Status::SUCCESS
}
