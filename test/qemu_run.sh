#!/bin/sh

usage() {
    echo "Usage: $0 [arch] efi_path"
}

check_ovmf() {
    OVMF_PREBUILT_TAG='edk2-stable202211-r1'
    base_url="https://github.com/rust-osdev/ovmf-prebuilt/releases/download"
    # need to download ovmf releases?
    if [ ! -d ovmf ]; then
        url="$(printf "%s/%s/%s-bin.tar.xz" "$base_url" "$OVMF_PREBUILT_TAG" "$OVMF_PREBUILT_TAG")"
        echo "Downloading ovmf releases from $url"
        mkdir ovmf
        wget -q -O- "$url" | tar -xJ -C ovmf --strip-components=1
    fi
}

if [ "$#" != 2 ]; then
    usage
    exit 1
fi

# default options
arch='x64'
qemu_system='qemu-system-x86_64'

if [ "$1" = 'arm' ] || [ "$1" = 'arm64' ] || [ "$1" = 'aarch64' ]; then
    arch='aarch64'
    qemu_system='qemu-system-aarch64'
fi

boot='esp/efi/boot'
test_dir="$(dirname "$0")"
efi="$(realpath --relative-to="$test_dir" "$2")"

cd "$test_dir" || exit 1
mkdir -p "$boot"
cp "$efi" "${boot}/bootx64.efi"

check_ovmf
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

