# lboot

**A Lightweight Bootloader** implemented in Rust.

## Compilation

```console
$ cargo build --target x86_64-unknown-uefi
```

The executable can be found at `target/x86_64-unknown-uefi/debug/lboot.efi`.

## Usage

> Note: This program is still in the testing phase. It is recommended to run it in the QEMU emulator.

### Configuration File

The configuration file is written in a subset of [TOML](https://toml.io/) syntax and should be placed in the EFI partition, for example, `/boot/lboot.toml`.

> Note: Syntax features like comments and escape characters are not supported.

```toml
[[entry]]
name = "Arch Linux"
vmlinux = '\vmlinuz-linux'
param = 'initrd=\initramfs-linux.img root=UUID=48b884eb-2b80-xxxx-xxxx-xxxxxxxxxxxx rw  loglevel=3 quiet'
```

- `name`: The name of this boot entry, which can be empty.
- `vmlinux`: The path to the kernel executable file used for booting. It should use absolute paths and `\` as the path separator.
- `param`: The boot parameters for the Linux kernel.

### QEMU Emulation

> Ensure that you are using the QEMU software package with a graphical interface.

First, edit the configuration file [lboot.qemu.toml](lboot.qemu.toml) and prepare files such as bzImage, initrd, rootfs, etc., for Linux boot.

Create the directory structure for emulation and place the configuration file in the specified location:

```console
$ mkdir -p test/esp/efi/boot
$ cp path/to/lboot.qemu.toml test/esp/efi/boot/lboot.toml
$ cp path/to/bzImage test/esp/efi/boot/
$ cp path/to/initramfs-linux.img test/esp/efi/boot
```

QEMU will recognize the `esp` directory as a FAT drive partition and automatically boot to the `bootx64.efi` file inside it.

The `qemu_run.sh` script in the `test` directory copies the executable file to `esp/efi/boot/bootx64.efi` and starts the QEMU emulation with the appropriate command-line parameters:

```console
./test/qemu_run.sh
```

### Installation on the Operating System

Assuming the compiled `lboot.efi` is placed in `/boot/EFI/lboot/lboot.efi`, and the EFI system partition is on the hard disk `/dev/sda`:

```console
$ sudo ./lboot-install.sh /dev/sda /boot/EFI/lboot/lboot.efi
```

Also, place the configuration file in `/boot/lboot.toml`.

### Boot Menu

This bootloader provides a menu interface similar to GRUB2, with the `>` indicating the currently selected boot entry:

```text
> Linux 6.5.5@[efi\boot\bzImage.efi] -- initrd=efi\boot\initramfs-linux.img
  Linux 6.4.16@[efi\boot\bzImage6.4.16.efi] -- initrd=efi\boot\initramfs-linux.img
  Linux 6.1.55@[efi\boot\bzImage6.1.55.efi] -- initrd=efi\boot\initramfs-linux.img
```

Use `k` or `UP` to move up, `j` or `DOWN` to move down, and `RETURN` or `RIGHT` or `l` to select the highlighted boot entry.

If no selection is made within 3 seconds, the system will automatically boot into the first boot entry.
