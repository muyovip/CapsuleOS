# Integration Guide: Connecting Boot Infrastructure to Genesis Engine

This document explains how to integrate the bare-metal boot infrastructure with the actual Genesis Graph Engine implementation.

## Overview

The boot infrastructure in `boot/`, `initramfs/`, and `kernel/` is an **architectural specification** showing how CapsuleOS boots from GRUB to the GGE runtime. To create a working system, you must replace the stub components with real implementations.

## Integration Points

### 1. Genesis Graph Engine Runtime

**Current State:**
- Location: `initramfs/usr/bin/gge-runtime`
- Type: Shell script stub that echoes log messages
- Purpose: Demonstrates boot handoff interface

**Production Integration:**

```bash
# Step 1: Build the actual genesis_engine
cd workspace/genesis_engine
cargo build --release --target x86_64-unknown-linux-gnu

# Step 2: Copy binary to initramfs
cp target/release/genesis_engine ../initramfs/usr/bin/gge-runtime

# Step 3: Verify binary is executable
chmod +x ../initramfs/usr/bin/gge-runtime

# Step 4: Rebuild initramfs image
cd ../initramfs
find . | cpio -o -H newc | gzip > ../boot/initramfs.img
```

**Expected Interface:**

The GGE runtime must accept these command-line arguments from `/init`:

```bash
gge-runtime \
    --init-capsule <CID> \
    --log-hash <SHA256> \
    --deterministic \
    --log-mutations \
    --audit-mode
```

**Required Behavior:**
1. Accept Root Capsule CID as `--init-capsule`
2. Load capsule from CAS at `/mnt/cas/cid/<CID>`
3. Register capsule as initial graph state
4. Begin evaluation loop (ΔG until idle)
5. Become PID 1 (primordial process)
6. Never return to `/init` script

### 2. Capsule Loader (Cryptographic Verification)

**Current State:**
- Location: `initramfs/sbin/capsule-loader`
- Type: Shell script with simulated verification
- Purpose: Shows signature verification interface

**Production Integration:**

Replace shell script with Rust binary using `capsule_core`:

```rust
// capsule-loader/src/main.rs

use capsule_core::crypto::{PublicKey, Signature, verify_ed25519};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    match args[1].as_str() {
        "--verify" => {
            let path = &args[3];
            let data = fs::read(path)?;
            
            // Extract signature from capsule metadata
            let (signature_bytes, capsule_data) = extract_signature(&data);
            
            // Load root public key from genesis.cfg
            let pub_key = load_root_public_key()?;
            
            // Verify Ed25519 signature
            let sig = Signature::from_bytes(&signature_bytes)?;
            let pk = PublicKey::from_bytes(&pub_key)?;
            
            if verify_ed25519(&capsule_data, &sig, &pk) {
                println!("SUCCESS: Capsule signature verified");
                std::process::exit(0);
            } else {
                eprintln!("FATAL: Signature verification FAILED");
                std::process::exit(1);
            }
        },
        "--load" => {
            // Memory mapping logic using kernel syscalls
            load_capsule_to_memory(&args[3], &args[5])?;
            Ok(())
        },
        _ => {
            eprintln!("Usage: capsule-loader [--verify|--load] ...");
            std::process::exit(1);
        }
    }
}
```

**Build and Deploy:**

```bash
cd capsule-loader
cargo build --release --target x86_64-unknown-linux-gnu
cp target/release/capsule-loader ../initramfs/sbin/
```

### 3. Content-Addressed Storage (CAS) Mount

**Current State:**
- Location: `initramfs/sbin/cas-mount`
- Type: Shell script stub
- Purpose: Shows CAS mounting interface

**Production Integration Options:**

**Option A: IPFS Local Node**
```bash
#!/bin/sh
# Start IPFS daemon and mount via FUSE
ipfs daemon --init --offline &
sleep 2
ipfs mount /mnt/cas
```

**Option B: CAS Block Device**
```bash
#!/bin/sh
# Mount dedicated CAS partition
mount -t ext4 /dev/sda2 /mnt/cas
```

**Option C: Custom CAS Implementation**
```rust
// Implement custom FUSE filesystem for CAS
use fuse::{Filesystem, ReplyEntry, Request};

struct CasFilesystem {
    storage: HashMap<String, Vec<u8>>,
}

impl Filesystem for CasFilesystem {
    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // Lookup content by CID
        let cid = name.to_str().unwrap();
        if let Some(data) = self.storage.get(cid) {
            // Return inode for CID
        }
    }
    // ... implement other FUSE operations
}
```

### 4. Kernel Integration

**Current State:**
- Location: `kernel/src/main.rs`
- Type: Conceptual Rust code
- Purpose: Shows kernel initialization architecture

**Production Integration:**

1. **Assembly Entry Point** (`kernel/arch/x86_64/boot.asm`):
```asm
global _start
extern kmain

section .text
bits 32

_start:
    ; Multiboot entry point
    mov esp, stack_top
    
    ; Clear BSS section
    mov edi, __bss_start
    mov ecx, __bss_end
    sub ecx, edi
    xor eax, eax
    rep stosb
    
    ; Jump to Rust kmain
    push ebx  ; multiboot_info pointer
    call kmain
    
    ; Should never return
    hlt
```

2. **Linker Script** (`kernel/linker.ld`):
```ld
ENTRY(_start)

SECTIONS {
    . = 1M;
    
    .boot : {
        *(.multiboot)
    }
    
    .text : {
        *(.text)
    }
    
    .rodata : {
        *(.rodata)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        __bss_start = .;
        *(.bss)
        __bss_end = .;
    }
}
```

3. **Build Configuration** (`kernel/Cargo.toml`):
```toml
[build]
target = "x86_64-unknown-none"

[profile.release]
panic = "abort"
lto = true
opt-level = 3
```

## Testing Integration

### Local Testing with QEMU

```bash
# Build all components
./scripts/build-all.sh

# Run in QEMU
qemu-system-x86_64 \
    -kernel boot/capsuleos-kernel \
    -initrd boot/initramfs.img \
    -serial stdio \
    -nographic \
    -m 256M
```

### Bare-Metal Testing

```bash
# Install to USB drive (DANGEROUS - verify device first!)
sudo dd if=boot/capsuleos-kernel of=/dev/sdX bs=1M
sudo dd if=boot/initramfs.img of=/dev/sdX bs=1M seek=10

# Install GRUB
sudo grub-install --boot-directory=/mnt/boot /dev/sdX
sudo cp boot/grub/grub.cfg /mnt/boot/grub/
```

## Validation

### Expected Boot Log (Successful Integration)

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
LOG: ⊙₀ Loaded. Hash: a3f2b9c8e1d4f6a7...
STEP: Spawning Genesis Graph Engine (GGE) runtime...
GGE: Genesis Graph Engine starting up...
INFO: GenesisGraph initialized from Root Capsule
INFO: Entering evaluation loop (ΔG until idle)...
[GGE continues as PID 1]
```

### Troubleshooting

**Problem: GGE not starting**
- Check: Is `gge-runtime` the actual genesis_engine binary?
- Check: Are command-line args correct in `/init`?
- Check: Is CAS mounted with root capsule accessible?

**Problem: Signature verification fails**
- Check: Is root public key in `genesis.cfg` correct?
- Check: Is capsule signed with matching private key?
- Check: Is capsule-loader using real Ed25519 verification?

**Problem: CAS mount fails**
- Check: Is IPFS daemon running?
- Check: Is CAS block device formatted correctly?
- Check: Are permissions correct on `/mnt/cas`?

## Next Steps

1. **Implement GGE Binary** - Make genesis_engine accept boot arguments
2. **Replace Capsule Loader** - Use real capsule_core verification
3. **Integrate CAS Backend** - Choose IPFS, block device, or custom
4. **Build Kernel** - Complete bare-metal implementation
5. **Test End-to-End** - Verify full boot sequence in QEMU
6. **Deploy to Hardware** - Test on bare metal

## References

- Genesis Engine Implementation: `workspace/genesis_engine/`
- Capsule Core Crypto: `workspace/capsule_core/src/crypto.rs`
- Boot Specification: `boot/README.md`
- Multiboot Spec: https://www.gnu.org/software/grub/manual/multiboot/
