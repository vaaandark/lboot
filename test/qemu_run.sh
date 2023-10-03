#!/bin/sh
set -eux

# default options
target='x86_64-unknown-uefi'
arch='x86_64'
qemu_system='qemu-system-x86_64'

if [ "$#" -eq 1 ] ; then
    if [ "$1" = 'arm' ] || [ "$1" = 'arm64' ] || [ "$1" = 'aarch64' ]; then
        arch='aarch64'
        target='aarch64-unknown-uefi'
        qemu_system='qemu-system-aarch64'
    fi
fi

boot='esp/efi/boot'
target_dir="../target/${target}/debug"
efi="${target_dir}/lboot.efi"

cd "$(dirname "$0")"
mkdir -p "$boot"
cp "$efi" "${boot}/bootx64.efi"

code="ovmf/${arch}/code.fd"
vars="ovmf/${arch}/vars.fd"
vars_read_only='on'
target_specific="-machine q35 -smp 4 -vga std --enable-kvm \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04"
if [ "$arch" = 'aarch64' ]; then
    tmp_var="$(mktemp)"
    cp "$vars" "$tmp_var"
    vars="$tmp_var"
    vars_read_only='off'
    target_specific="-machine virt -cpu cortex-a72 \
        -device virtio-gpu-pci"
fi

"$qemu_system" \
    -m 1G \
    -serial stdio \
    -nodefaults \
    -device virtio-rng-pci \
    -boot menu=on,splash-time=0 \
    $target_specific \
    -drive if=pflash,format=raw,readonly=on,file="$code" \
    -drive if=pflash,format=raw,readonly="$vars_read_only",file="$vars" \
    -drive format=raw,file=fat:rw:esp

