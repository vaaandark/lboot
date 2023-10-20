#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use lboot::config::{BoxedCStr16, Entry};
use lboot::menu::select_in_menu;
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
    info!("Testing grub2-like menu now...");
    let entries: Vec<Entry> = (1u8..10)
        .into_iter()
        .map(|i| {
            let ch16: u16 = (i + b'0') as u16;
            let s = vec![ch16, 0];
            Entry {
                name: Some(BoxedCStr16::new(s)),
                vmlinux: None,
                param: None,
            }
        })
        .collect();
    let selected = select_in_menu(&mut st, &entries).unwrap();
    println!("Entry {} is selected!", selected.name.as_ref().unwrap());
    st.boot_services().stall(3_000_000);
    shutdown(st);
}
