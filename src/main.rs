#![no_main]
#![no_std]

extern crate alloc;

use lboot::{config::Config, image::start_kernel_image, menu::select_in_menu};
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
    let config_list = [
        cstr16!(r"\efi\boot\lboot.toml"),
        cstr16!(r"\lboot.toml"),
        cstr16!(r"\efi\lboot.toml"),
        cstr16!(r"\boot\lboot.toml"),
        cstr16!(r"\boot\efi\lboot.toml"),
    ];
    let config = config_list
        .iter()
        .find_map(|filename| Config::load_from_file(st.boot_services(), filename).ok())
        .unwrap();
    let entries = config.parse().unwrap();
    let entry = select_in_menu(&mut st, &entries).unwrap();
    info!("Loading {}", entry);
    start_kernel_image(image_handle, st.boot_services(), entry).unwrap();
    Status::SUCCESS
}
