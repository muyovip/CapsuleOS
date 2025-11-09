#!/bin/bash
set -e

echo "Forging CapsuleOS ISO..."

mkdir -p iso/boot
cp target/x86_64-unknown-none/release/capsuleos iso/boot/kernel.elf
cp limine.cfg iso/boot/

# Install Limine
./limine bios-install iso/boot/limine-bios.sys

# Build ISO
xorriso -as mkisofs \
  -b limine-bios.sys \
  -no-emul-boot \
  -boot-load-size 4 \
  -boot-info-table \
  --efi-boot limine-uefi-cd.bin \
  -efi-boot-part --efi-boot-image \
  iso -o capsuleos.iso

echo "capsuleos.iso forged. Download from Files."
