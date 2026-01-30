# wflos - Rust Microkernel OS

A Rust-based microkernel operating system with capability-based security foundations, developed on Apple Silicon M1 for x86_64 targets.

## Current Status: Phase 3 Complete ✅

### Implemented Features

**Phase 1: Minimal Boot**
- ✅ **Cross-compilation infrastructure** (ARM64 macOS → x86_64 kernel)
- ✅ **Limine bootloader integration** (v8.x protocol)
- ✅ **VGA text mode driver** with spinlock-based safe access
- ✅ **Serial port driver** (COM1) for debugging
- ✅ **Memory-safe architecture** (no `static mut`, volatile MMIO)
- ✅ **Higher-half kernel** at 0xffffffff80000000
- ✅ **Boot successfully** in QEMU with display output

**Phase 2: Memory Management & Core Services**
- ✅ **GDT (Global Descriptor Table)** - 5 segments for kernel/user code/data
- ✅ **IDT (Interrupt Descriptor Table)** - 256 entries with exception handlers
- ✅ **Exception handlers** - Divide-by-zero, page fault, GPF, double fault, etc.
- ✅ **Physical frame allocator** - Bitmap-based, manages 4KB frames
- ⏸️ **Heap allocator** - Ready but deferred (needs page tables)

**Phase 3: Keyboard Input** ⭐ NEW
- ✅ **Ring buffer** - Hardware-agnostic circular buffer for input buffering
- ✅ **PIC configuration** - Remaps IRQs to vectors 32-47
- ✅ **PS/2 keyboard driver** - Reads scan codes from keyboard controller
- ✅ **Interrupt handling** - IRQ1 keyboard interrupts working
- ✅ **Scan code translation** - Converts scan codes to ASCII (US layout)

### Verification

```bash
# Build and verify
make kernel
file target/x86_64-unknown-none/debug/kernel
# Should show: ELF 64-bit LSB executable, x86-64

# Create bootable ISO
make iso

# Run in QEMU
make run
```

**Expected output:**
```
Serial port initialized
HHDM offset: 0xffff800000000000
VGA initialized
wflos - Rust Microkernel OS
Version 0.3.0 (Phase 3: Keyboard Input)
Initializing GDT...
  GDT loaded - segments already configured by bootloader
GDT loaded
Initializing IDT...
IDT loaded
Initializing PIC...
PIC initialized and remapped
Initializing keyboard...
Keyboard initialized
Enabling interrupts...
Interrupts enabled
Initializing frame allocator...
Frame allocator: 64243 total, 0 used, 64243 free

=== Phase 3 Complete ===
  - GDT initialized and loaded
  - IDT initialized with exception handlers
  - Frame allocator operational (64243 frames available)
  - PIC remapped (IRQs at vectors 32-47)
  - Keyboard driver ready (IRQ1)
  - Interrupts enabled
========================

Keyboard ready for input!
```

## Architecture

### Memory Safety Patterns

1. **No `static mut`** - All mutable globals use `Spinlock<T>`:
   ```rust
   static VGA_WRITER: Spinlock<VgaBuffer> = Spinlock::new(...);
   ```

2. **Volatile MMIO** - Hardware registers use volatile operations:
   ```rust
   ptr::write_volatile(self, value);
   ptr::read_volatile(self);
   ```

3. **Strict Provenance** - Physical addresses accessed via HHDM:
   ```rust
   let virt_ptr = (hhdm_offset + phys_addr) as *mut T;
   ```

### Cross-Compilation Setup

**Critical:** The project separates host (ARM64) and target (x86_64) architectures:

- **Kernel**: Built for `x86_64-unknown-none` using `rust-lld`
- **Limine utility**: Built for native ARM64 macOS
- **Cargo configuration**: Forces correct linker and enables `build-std`

See `.cargo/config.toml` and `x86_64-unknown-none.json` for details.

## Project Structure

```
wflos/
├── kernel/
│   ├── src/
│   │   ├── main.rs          # Entry point (_start)
│   │   ├── limine.rs        # Bootloader protocol
│   │   ├── drivers/
│   │   │   ├── vga.rs       # VGA text mode
│   │   │   └── serial.rs    # Serial debugging
│   │   └── sync/
│   │       └── spinlock.rs  # No-std spinlock
│   ├── linker.ld            # Higher-half linker script
│   └── Cargo.toml
├── shared/
│   └── src/                 # Hardware-agnostic utilities
├── x86_64-unknown-none.json # Custom target spec
├── limine.conf              # Bootloader config
└── Makefile                 # Build orchestration
```

## Development Workflow

### Building

```bash
# Build kernel only
make kernel

# Build Limine utility
make limine-utility

# Build bootable ISO
make iso

# Run in QEMU
make run

# Clean build artifacts
make clean

# Verify cross-compilation setup
make verify
```

### Testing

```bash
# Host-based unit tests (runs on macOS ARM64)
make test-host

# Integration tests (runs in QEMU)
make test-integration

# All tests
make test
```

## Requirements

- **Rust nightly** toolchain
- **QEMU** (qemu-system-x86_64)
- **xorriso** for ISO creation
- **git** for Limine submodule
- **Make** and standard build tools

### Installation

```bash
# Install Rust nightly
rustup toolchain install nightly

# Install dependencies (macOS)
brew install qemu xorriso
```

## Roadmap

### ~~Phase 2: Memory Management & Core Services~~ ✅ COMPLETE
- [x] Global Descriptor Table (GDT)
- [x] Interrupt Descriptor Table (IDT)
- [x] Exception handlers
- [x] Physical frame allocator
- [ ] Virtual memory paging (deferred)
- [ ] Kernel heap allocator (deferred)

### ~~Phase 3: Keyboard Input~~ ✅ COMPLETE
- [x] PS/2 keyboard driver
- [x] Interrupt handling (PIC configuration)
- [x] Ring buffer for input
- [x] Scan code → ASCII translation

### Phase 4: Command-Line Interface
- [ ] Shell REPL (Read-Eval-Print Loop)
- [ ] Command parser
- [ ] Built-in commands:
  - `help` - List commands
  - `clear` - Clear screen
  - `echo <text>` - Print text
  - `meminfo` - Memory statistics
  - `halt` - Stop system
  - `version` - Kernel version

### Phase 5: Testing Infrastructure
- [ ] Host-based unit tests
- [ ] QEMU integration tests
- [ ] Automated CI/CD

### Future (Post-MVP)
- Capability-based IPC
- Userspace processes
- Drivers as isolated processes
- NVMe storage driver
- Network stack
- Filesystem

## Technical Details

### Cross-Compilation on M1

The Makefile solves the M1 cross-compilation challenge:

1. **Kernel**: Uses `cargo build --target x86_64-unknown-none.json`
   - Forces `rust-lld` linker
   - Produces ELF x86-64 binary

2. **Limine utility**: Uses native macOS toolchain
   - Clones v8.x-binary branch (pre-built blobs)
   - Compiles only host utility for ARM64

3. **Verification**: `make verify` checks architecture of both components

### Limine Bootloader

wflos uses the Limine protocol for boot:

- **HHDM Request**: Maps all physical memory to higher-half
- **Memory Map**: Provides usable RAM regions
- **Kernel Address**: Reports kernel load location
- **Framebuffer**: For future graphics support

See `kernel/src/limine.rs` for protocol implementation.

### VGA Driver

Text mode at 0xB8000 (80x25 characters):

- Color code: 4-bit background + 4-bit foreground
- Character format: ASCII byte + color byte
- Volatile operations prevent compiler optimization
- Protected by `Spinlock` for safe concurrent access

### Serial Driver

COM1 at 0x3F8 (38400 baud):

- Initialized early for boot debugging
- Uses x86_64 `in`/`out` instructions
- Implements `fmt::Write` trait
- Macros: `serial_print!()`, `serial_println!()`

## Resources

- **Architecture Guide**: `Docs/Building Rust Microkernel OS on Mac.md`
- **Limine Protocol**: https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md
- **OSDev Wiki**: https://wiki.osdev.org/

## License

MIT (to be added)

## Contributing

This is an educational project. Contributions welcome!

---

**Built with Rust 🦀 on Apple Silicon M1 for x86_64**
