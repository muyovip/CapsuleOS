# CapsuleOS Boot Infrastructure - Work Order 12

This directory contains the foundational boot layer infrastructure for CapsuleOS Hypervisor Protocol Layer 0.

## Overview

CapsuleOS implements a bare-metal boot sequence that establishes cryptographic trust from the bootloader to the Genesis Graph Engine (GGE) runtime. The boot process follows this sequence:

```
GRUB → Kernel → Initramfs → /init → CAS Mount → ⊙₀ Verification → GGE Runtime
```

## Directory Structure

```
boot/
├── grub/
│   └── grub.cfg           # GRUB bootloader configuration
├── capsuleos-kernel       # Bare-metal kernel binary (compiled from kernel/)
└── initramfs.img          # Initial ramdisk with boot environment

initramfs/
├── init                   # Main boot sequence script (PID 1)
├── bin/                   # Basic utilities (sh, etc.)
├── sbin/                  # System utilities
│   ├── cas-mount          # Content-Addressed Storage mount utility
│   ├── capsule-loader     # Cryptographic capsule verification/loading
│   └── hash-utility       # SHA-256 hash computation for boot audit
├── etc/
│   └── genesis.cfg        # Storage and runtime configuration
├── usr/bin/
│   └── gge-runtime        # Genesis Graph Engine executable
└── mnt/cas/               # CAS mount point

kernel/
└── src/
    ├── main.rs            # Kernel entry point (kmain)
    ├── capsule/
    │   └── loader.rs      # Capsule verification and loading
    ├── drivers/
    │   └── serial.rs      # Serial console for boot logging
    ├── mmu.rs             # Memory management (paging, virtual memory)
    └── vfs.rs             # Virtual filesystem (initramfs support)
```

## Boot Sequence

### Phase 1: GRUB Bootloader

GRUB loads the kernel and initramfs into memory and jumps to the kernel entry point.

**Configuration:** `boot/grub/grub.cfg`

```grub
menuentry 'CapsuleOS' {
    multiboot /boot/capsuleos-kernel root=/dev/ram0 rw
    initrd /boot/initramfs.img
    boot
}
```

### Phase 2: Kernel Initialization

The kernel performs low-level hardware initialization.

**Code:** `kernel/src/main.rs` - `kmain()` function

1. Initialize serial port for logging
2. Set up MMU and paging (virtual memory)
3. Initialize interrupt handling (IDT, exceptions)
4. Mount initramfs as root filesystem
5. Execute `/init` script (transition to userspace)

### Phase 3: Initramfs Boot Sequence

The `/init` script executes the main boot sequence.

**Script:** `initramfs/init`

1. **Hardware Verification** - Confirm kernel initialized successfully
2. **CAS Mount** - Mount Content-Addressed Storage via IPFS/CAS
3. **Root Capsule Loading**:
   - Locate ⊙₀ capsule by CID
   - Verify Ed25519 signature (Chain of Trust)
   - Load capsule into memory at 0x40000000
   - Compute and log deterministic hash
4. **GGE Hand-off** - Execute GGE runtime as PID 1

### Phase 4: GGE Runtime

The Genesis Graph Engine becomes the primordial process.

**Binary:** `initramfs/usr/bin/gge-runtime`

1. Register ⊙₀ as initial graph state
2. Load rewrite rules from capsule manifest
3. Begin parallel pattern matching evaluation
4. Establish CapsuleOS runtime environment

## Configuration

### Genesis Configuration (`initramfs/etc/genesis.cfg`)

```ini
[storage]
type = ipfs_local              # CAS backend: ipfs_local, ipfs_remote, cas_block_device
endpoint = /mnt/cas            # CAS mount point
root_cid = QmRootCapsule...    # Root Capsule ⊙₀ Content Identifier
root_public_key = ed25519:...  # Ed25519 public key for verification

[modules]
gge_path = /usr/bin/gge-runtime
gge_flags = --deterministic --log-mutations --audit-mode

[boot]
deterministic = true           # Reproducible boot sequence
log_device = /dev/kmsg         # Kernel message buffer
mount_timeout_ms = 5000
verification_mode = strict     # strict | permissive
```

## Cryptographic Chain of Trust

The boot sequence establishes an unbroken cryptographic chain:

1. **GRUB** - Trusted bootloader (secure boot)
2. **Kernel** - Loaded and verified by GRUB
3. **Initramfs** - Loaded and verified by kernel
4. **Root Capsule (⊙₀)** - Ed25519 signature verified against embedded public key
5. **GGE Runtime** - Loaded from verified ⊙₀ capsule

At each step, cryptographic verification ensures the integrity of the next component.

## Building and Testing

### Build Kernel

```bash
cd kernel
cargo build --release
cp target/release/capsuleos-kernel ../boot/
```

### Build Initramfs

```bash
cd initramfs
find . | cpio -o -H newc | gzip > ../boot/initramfs.img
```

### Test with QEMU

```bash
qemu-system-x86_64 \
    -kernel boot/capsuleos-kernel \
    -initrd boot/initramfs.img \
    -serial stdio \
    -nographic
```

### Expected Boot Log

```
CapsuleOS Kernel: kmain started.
INFO: MMU/Paging initialized.
INFO: Initial VFS mounted (initramfs).
--- CapsuleOS Hypervisor Protocol Layer 0 (Bare-Metal Boot) ---
INFO: Low-level hardware initialization complete (CPU, memory, MMU).
STEP: Attempting early mount of Content-Addressed Storage...
SUCCESS: CAS mounted at /mnt/cas.
STEP: Loading Root Capsule (⊙₀) from /mnt/cas/cid/QmRootCapsule...
INFO: Root Capsule signature verified.
LOG: ⊙₀ Loaded. Hash: 0xAFFECAFEBEEFDEAD...
STEP: Spawning Genesis Graph Engine (GGE) runtime...
GGE: Genesis Graph Engine starting up...
```

## Security Considerations

1. **Root Public Key** - Embedded in kernel at compile time
2. **Signature Verification** - All capsules verified before execution
3. **Deterministic Hashing** - Boot audit trail for reproducibility
4. **Memory Protection** - MMU enforces kernel/user separation
5. **Audit Logging** - All mutations logged to `/dev/kmsg`

## Implementation Notes

**IMPORTANT: This is an ARCHITECTURAL SPECIFICATION, not production code.**

This implementation demonstrates the **DESIGN and ARCHITECTURE** of a bare-metal operating system boot sequence for CapsuleOS. The files provide a complete specification of how the boot process would work, including:

- Boot sequence flow (GRUB → Kernel → Initramfs → GGE)
- Cryptographic chain of trust architecture
- Kernel initialization phases
- Boot script logic and error handling
- Configuration format and parameters
- Expected boot log output

### Stub Components (To Be Replaced in Production)

1. **gge-runtime** (`initramfs/usr/bin/gge-runtime`)
   - Current: Shell script stub that echoes log messages
   - Production: Compiled `genesis_engine` binary from workspace/genesis_engine/
   - Integration: Copy `genesis_engine/target/release/genesis_engine` to this path

2. **capsule-loader** (`initramfs/sbin/capsule-loader`)
   - Current: Shell script with simulated verification
   - Production: Binary using `capsule_core` crypto primitives for real Ed25519 verification
   - Integration: Implement using `capsule_core::crypto::verify()`

3. **Kernel modules** (`kernel/src/`)
   - Current: Conceptual Rust code with documented interfaces
   - Production: Full bare-metal implementation with assembly entry point

### Production Implementation Requirements

A production implementation would require:

- Complete assembly entry point (`_start`)
- Full MMU/paging implementation (x86_64 page tables)
- Real IPFS/CAS integration
- Complete interrupt handling (IDT, ISR)
- Full VFS implementation (cpio parsing, tmpfs)
- Hardware timer for deterministic timestamps
- Secure boot integration (UEFI Secure Boot)

## References

- GRUB Multiboot Specification: https://www.gnu.org/software/grub/manual/multiboot/
- x86_64 Paging: Intel® 64 and IA-32 Architectures Software Developer's Manual
- Ed25519 Signatures: RFC 8032
- IPFS: https://docs.ipfs.tech/
- Linux Kernel Boot Process: https://www.kernel.org/doc/html/latest/

## License

Copyright (c) 2025 CapsuleOS Project
