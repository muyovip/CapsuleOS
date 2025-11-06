#!/usr/bin/env bash
set -euo pipefail

# Locations
ROOT="$(pwd)"
WORKSPACE_DIR="$ROOT/workspace"
INITRAMFS_DIR="$ROOT/boot/initramfs_root"
OUT_INIT="$ROOT/boot/initramfs.cpio.gz"

echo "Cleaning..."
rm -rf "$INITRAMFS_DIR"
mkdir -p "$INITRAMFS_DIR/bin" "$INITRAMFS_DIR/sbin" "$INITRAMFS_DIR/etc" "$INITRAMFS_DIR/capsules" "$INITRAMFS_DIR/var/gge"

# Copy busybox for basic shell utilities (assumes busybox installed)
BUSYBOX=$(which busybox || true)
if [ -z "$BUSYBOX" ]; then
  echo "busybox not found in PATH. Please install busybox."
  exit 1
fi
cp "$BUSYBOX" "$INITRAMFS_DIR/bin/busybox"
chmod +x "$INITRAMFS_DIR/bin/busybox"
# Create many applets
for cmd in sh ls echo cat mkdir mount umount sleep grep dd rm cp mv ln; do
  ln -sf /bin/busybox "$INITRAMFS_DIR/bin/$cmd"
done

# Copy Genesis config
mkdir -p "$INITRAMFS_DIR/etc"
cp "$ROOT/boot/initramfs/etc/genesis.cfg" "$INITRAMFS_DIR/etc/genesis.cfg"

# Copy capsules and scene files (pre-bundled)
mkdir -p "$INITRAMFS_DIR/capsules"
cp -r "$ROOT/tests" "$INITRAMFS_DIR/"

# Copy gge binary
GGE_BIN="$WORKSPACE_DIR/target/release/gge"
if [ ! -f "$GGE_BIN" ]; then
  echo "gge binary not found. Building workspace..."
  (cd "$WORKSPACE_DIR" && cargo build --release)
fi
cp "$GGE_BIN" "$INITRAMFS_DIR/sbin/gge"
chmod +x "$INITRAMFS_DIR/sbin/gge"

# init script
cp "$ROOT/boot/initramfs/integration_init" "$INITRAMFS_DIR/init"
chmod +x "$INITRAMFS_DIR/init"

# Create cpio archive
pushd "$INITRAMFS_DIR" >/dev/null
find . | cpio -o -H newc | gzip -9 > "$OUT_INIT"
popd >/dev/null

echo "Created initramfs: $OUT_INIT"
