#!/usr/bin/env bash
set -euo pipefail

KERNEL=${KERNEL:-/boot/vmlinuz}  # override if needed
INITRD=boot/initramfs.cpio.gz

if [ ! -f "$KERNEL" ]; then
  echo "Kernel not found at $KERNEL. Please specify KERNEL env var to point to a bzImage."
  exit 1
fi

qemu-system-x86_64 -m 1024 -kernel "$KERNEL" -initrd "$INITRD" \
  -append "console=ttyS0 root=/dev/ram0 rw" \
  -nographic
