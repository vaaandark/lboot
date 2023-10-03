# lboot

A lightweight boot loader implemented in Rust.

## Compilation

```console
$ cargo build --target x86_64-unknown-uefi
```

This will produce an executable: `target/x86_64-unknown-uefi/debug/lboot.efi`.

## Usage

> The program is still in the testing phase. It is recommended to use qemu for simulation.

### Configuration File

The configuration file is written using a subset of the [TOML](https://toml.io/) syntax. An example configuration file, [lboot.toml](lboot.toml), **should be placed in `/efi/boot/lboot.toml`**.

> Comments and escape characters are not supported.

```toml
[[entry]]
name = 'Linux 6.5.5'
vmlinux = 'efi\boot\bzImage'
param = 'initrd=efi\boot\initramfs-linux.img'
```

- `name`: The name of the boot item, can be empty.
- `vmlinux`: The path to the kernel executable file used for booting. Should be an absolute path using `\` as the path separator.
- `param`: The boot parameters for the Linux kernel.

### qemu Simulation

> A qemu software package with a graphical interface should be used.

First, edit the configuration file and prepare the bzImage, initrd, rootfs, and other files required for Linux boot.

Create a directory for simulation and place the configuration file in the specified location:

```console
$ mkdir -p test/esp/efi/boot
$ cp path/to/lboot.toml test/esp/efi/boot/
$ cp path/to/bzImage test/esp/efi/boot/
$ cp path/to/initramfs-linux.img test/esp/efi/boot
```

qemu will recognize the `esp` directory as a FAT drive partition and automatically boot into the `bootx64.efi` file inside it.

The `qemu_run.sh` script in the `test` directory copies the executable file to `esp/efi/boot/bootx64.efi` and starts the qemu simulation with the appropriate command line parameters.

```console
./test/qemu_run.sh
```

### Boot Menu

A menu interface similar to grub2 is used, where `>` indicates the selected boot item:

```text
> Linux 6.5.5@[efi\boot\bzImage.efi] -- initrd=efi\boot\initramfs-linux.img
  Linux 6.4.16@[efi\boot\bzImage6.4.16.efi] -- initrd=efi\boot\initramfs-linux.img
  Linux 6.1.55@[efi\boot\bzImage6.1.55.efi] -- initrd=efi\boot\initramfs-linux.img
```

`k` or `UP` to move up, `j` or `DOWN` to move down, `RETURN` or `RIGHT` or `l` to select the highlighted boot item.

If no selection is made within 3 seconds, the first boot item is automatically selected.
