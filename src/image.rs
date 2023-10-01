//! Load and start kernel image.

extern crate alloc;

use alloc::vec::Vec;
use uefi::data_types::CStr16;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;
use uefi::Handle;
use uefi::{
    prelude::BootServices,
    proto::device_path::{
        build::{media::FilePath, DevicePathBuilder},
        DevicePath, DeviceSubType, DeviceType, LoadedImageDevicePath,
    },
};

use crate::config::Entry;
use crate::error::{LbootError, Result};

fn get_kernel_device_path<'a>(
    bt: &BootServices,
    path_name: &CStr16,
    storage: &'a mut Vec<u8>,
) -> Result<&'a DevicePath> {
    let loaded_image_device_path = bt
        .open_protocol_exclusive::<LoadedImageDevicePath>(bt.image_handle())?;

    let mut builder = DevicePathBuilder::with_vec(storage);
    for node in loaded_image_device_path.node_iter() {
        if node.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
            break;
        }
        builder = builder
            .push(&node)
            .map_err(|_| LbootError::GenerateImagePathError)?;
    }
    builder = builder
        .push(&FilePath { path_name })
        .map_err(|_| LbootError::GenerateImagePathError)?;
    builder
        .finalize()
        .map_err(|_| LbootError::GenerateImagePathError)
}

/// Start a kernel image by a giving entry.
pub fn start_kernel_image(image_handle: Handle, bt: &BootServices, entry: &Entry) -> Result<()> {
    let mut storage = Vec::new();
    let path_name = entry.vmlinux.as_ref().ok_or(LbootError::WrongEntry)?.str;
    let kernel_image_path = get_kernel_device_path(bt, path_name, &mut storage)?;
    let kernel_image_handle = bt
        .load_image(
            image_handle,
            LoadImageSource::FromDevicePath {
                device_path: kernel_image_path,
                from_boot_manager: false,
            },
        )
        .map_err(|_| LbootError::CannotLoadImageIntoMemory)?;
    let mut kernel_loaded_image = bt
        .open_protocol_exclusive::<LoadedImage>(kernel_image_handle)?;
    if let Some(options) = &entry.param {
        let load_options = options.str;
        unsafe {
            kernel_loaded_image.set_load_options(
                load_options.as_ptr().cast(),
                load_options.num_bytes() as u32,
            );
        }
    }

    bt.start_image(kernel_image_handle)?;

    Ok(())
}
