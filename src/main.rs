#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::build::media::FilePath;
use uefi::proto::device_path::build::DevicePathBuilder;
use uefi::proto::device_path::{DevicePath, DeviceSubType, DeviceType, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;
use uefi::{
    entry,
    table::{Boot, SystemTable},
    Handle,
};

#[entry]
fn main(image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    let bt = st.boot_services();
    info!("Start booting...");
    let mut storage = Vec::new();
    let kernel_image_path = get_kernel_device_path(bt, &mut storage);
    let kernel_image_handle = bt
        .load_image(
            image_handle,
            LoadImageSource::FromDevicePath {
                device_path: kernel_image_path,
                from_boot_manager: false,
            },
        )
        .expect("failed to load kernel");
    let mut kernel_loaded_image = bt
        .open_protocol_exclusive::<LoadedImage>(kernel_image_handle)
        .expect("failed to open LoadedImage protocol");
    let load_options = cstr16!(r"initrd=efi\boot\initramfs-linux.img");
    unsafe {
        kernel_loaded_image.set_load_options(
            load_options.as_ptr().cast(),
            load_options.num_bytes() as u32,
        );
    }

    bt.start_image(kernel_image_handle).expect("failed to launch kernel");

    Status::SUCCESS
}

fn get_kernel_device_path<'a>(bt: &BootServices, storage: &'a mut Vec<u8>) -> &'a DevicePath {
    let loaded_image_device_path = bt
        .open_protocol_exclusive::<LoadedImageDevicePath>(bt.image_handle())
        .expect("failed to open LoadedImageDevicePath protocol");

    let mut builder = DevicePathBuilder::with_vec(storage);
    for node in loaded_image_device_path.node_iter() {
        if node.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
            break;
        }
        builder = builder.push(&node).unwrap();
    }
    builder = builder
        .push(&FilePath {
            path_name: cstr16!(r"efi\boot\bzImage.efi"),
        })
        .unwrap();
    builder.finalize().unwrap()
}

