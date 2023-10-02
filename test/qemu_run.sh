#!/bin/sh
set -eux

boot='esp/efi/boot'
target_dir='../target/x86_64-unknown-uefi/debug'
default_bz='/boot/vmlinuz-linuz'

efi="${target_dir}/lboot.efi"
if [ "$#" -eq 1 ] && [ -n "$1" ]; then
    efi="${target_dir}/examples/$1.efi"
fi

cd "$(dirname "$0")"
mkdir -p "$boot"
cp "$efi" "${boot}/bootx64.efi"
if [ ! -f "${boot}/bzImage.efi" ]; then
    cp "$default_bz" esp/efi/boot/bzImage.efi
fi

qemu-system-x86_64 -enable-kvm \
    -m 1G \
    -serial stdio \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp

