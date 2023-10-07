#!/bin/sh

if [ "$#" -eq 1 ] && [ "$1" = '-h' ]; then
    echo "$0: usage: {DISK} {EFI_PATH}"
    echo "    DISK: a disk containing bootloader"
    echo "    EFI_PATH: the lboot excutable path"
    exit 0
fi

if [ ! "$#" -eq 2 ]; then
    echo 'Expect a disk containing bootloader and the lboot excutable path'
    exit 1
fi

disk="$1"
path="$2"
esp_info="$(mount | grep vfat | grep "$disk")"
esp_partition="$(echo "$esp_info" | cut -d' ' -f1)"
esp_mount_point="$(echo "$esp_info" | cut -d' ' -f3)"
printf "Please examine your ESP partition:\n"
printf "%s\n" "$esp_info"
printf "Continue? (y/n)"
read -r answer
if [ "$answer" = "n" ] || [ "$answer" = "N" ]; then
    exit 1
fi

esp_path="$(realpath --relative-to="$esp_mount_point" "$path")"
esp_path="/$esp_path"
efibootmgr --create --disk "$esp_partition" --loader "$esp_path" --label 'lboot' --unicode
