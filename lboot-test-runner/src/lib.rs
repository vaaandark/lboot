#![no_main]
#![no_std]

use log::info;
use uefi::table::{SystemTable, Boot};

#[allow(unused)]
pub fn shutdown(mut st: SystemTable<Boot>) -> ! {
    // Get our text output back.
    st.stdout().reset(false).unwrap();

    info!("Testing complete, shutting down...");

    let (st, _iter) = st.exit_boot_services();

    #[cfg(target_arch = "x86_64")]
    {
        // Prevent unused variable warning.
        let _ = st;

        use qemu_exit::QEMUExit;
        let custom_exit_success = 3;
        let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
        qemu_exit_handle.exit_success();
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Shut down the system
        use uefi::Status;
        let rt = unsafe { st.runtime_services() };
        rt.reset(
            uefi::table::runtime::ResetType::SHUTDOWN,
            Status::SUCCESS,
            None,
        );
    }
}
