# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**wflos** is a Rust-based microkernel operating system with capability-based security foundations. The project is built on Apple Silicon M1 macOS but targets x86_64 architecture, requiring careful cross-compilation handling.

Current Status: MVP Complete (Phase 4) - Interactive shell with keyboard input, memory management, interrupt handling, and basic commands.

## Build Commands

### Essential Commands

```bash
# Build kernel only (x86_64 target)
make kernel

# Build bootable ISO (kernel + Limine bootloader)
make iso

# Run in QEMU with VGA display and serial output
make run

# Clean all build artifacts
make clean

# Verify cross-compilation setup
make verify
```

### Testing Commands

```bash
# Run host-based unit tests (ARM64)
make test-host

# Run integration tests in QEMU (x86_64)
make test-integration

# Run all tests
make test
```

### Manual QEMU Invocation

```bash
# Standard run with serial output
qemu-system-x86_64 -cdrom os.iso -serial stdio -no-reboot -no-shutdown -m 256M

# Debug with interrupt logging
qemu-system-x86_64 -cdrom os.iso -m 256M -d int -D debug.log
```

## Architecture Overview

### Cross-Compilation Strategy

The project handles ARM64 host → x86_64 target compilation:

1. **Kernel**: Built with `cargo +nightly build --target x86_64-unknown-none.json`
   - Uses `rust-lld` linker (specified in `.cargo/config.toml`)
   - Custom target spec disables red zone (`"disable-redzone": true`)
   - Links with `kernel/linker.ld` for higher-half kernel at `0xffffffff80000000`
   - Uses `build-std` to compile core/alloc from source

2. **Limine Bootloader**: Built for native ARM64 macOS
   - Cloned from `v8.x-binary` branch (contains pre-built BIOS/UEFI blobs)
   - Only the host utility is compiled (for creating bootable ISOs)
   - ISO contains both BIOS and UEFI boot support

3. **Verification**: Always check `file target/x86_64-unknown-none/debug/kernel` shows "ELF 64-bit LSB executable, x86-64"

### Memory Safety Patterns

**Critical**: This kernel follows strict safety patterns:

1. **NO `static mut`** - All mutable globals use `Spinlock<T>`:
   ```rust
   static VGA_WRITER: Spinlock<VgaBuffer> = Spinlock::new(...);
   ```

2. **Volatile MMIO** - Hardware registers require volatile operations:
   ```rust
   ptr::write_volatile(self, value);
   ptr::read_volatile(self);
   ```

3. **HHDM (Higher-Half Direct Map)** - Physical memory accessed via offset:
   ```rust
   let virt_ptr = (hhdm_offset + phys_addr) as *mut T;
   ```
   - HHDM offset provided by Limine bootloader
   - All physical memory mapped to higher-half virtual addresses

### Module Structure

```
kernel/src/
├── main.rs                    # Entry point (_start), boot sequence
├── limine.rs                  # Limine bootloader protocol requests
├── arch/x86_64/              # Architecture-specific code
│   ├── gdt.rs                # Global Descriptor Table (5 segments)
│   ├── idt.rs                # Interrupt Descriptor Table (256 entries)
│   ├── interrupts.rs         # Exception handlers (divide-by-zero, page fault, etc.)
│   └── pic/mod.rs            # Programmable Interrupt Controller (remaps IRQs to 32-47)
├── drivers/
│   ├── vga.rs                # VGA text mode (0xB8000, 80x25, spinlock-protected)
│   ├── serial.rs             # COM1 serial port (0x3F8, for debugging)
│   └── keyboard.rs           # PS/2 keyboard driver (IRQ1, scan code → ASCII)
├── memory/
│   ├── frame_allocator.rs    # Physical frame allocator (bitmap-based, 4KB frames)
│   └── heap.rs               # Heap allocator (ready but deferred, needs paging)
├── shell/
│   ├── mod.rs                # REPL main loop
│   ├── parser.rs             # Zero-copy command parsing with string slices
│   └── commands.rs           # Built-in commands (help, echo, version, meminfo, etc.)
└── sync/
    └── spinlock.rs           # No-std spinlock implementation
```

### Key Constraints

1. **`#![no_std]`** - No standard library, uses core + alloc only
2. **`#![no_main]`** - Entry point is `_start()` not `main()`
3. **Panic = abort** - No unwinding (set in Cargo.toml profiles)
4. **Stack-based shell** - No heap allocations in command processing (ring buffer uses fixed array)
5. **Single-threaded** - No preemptive multitasking (spinlocks never block)

## Development Patterns

### Adding Kernel Features

1. **Always initialize serial first** for debugging:
   ```rust
   drivers::serial::init();
   serial_println!("Debug message");
   ```

2. **Use macros for output**:
   - `println!()` / `print!()` → VGA display
   - `serial_println!()` / `serial_print!()` → Serial COM1

3. **Interrupt handlers must**:
   - Be `extern "x86-interrupt"` functions
   - Take specific argument types (see `arch/x86_64/interrupts.rs`)
   - Send EOI to PIC when done: `unsafe { pic::send_eoi(irq_number) }`

4. **Hardware I/O**:
   - x86 port I/O: Use `x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly}`
   - MMIO: Use `ptr::read_volatile()` / `ptr::write_volatile()`
   - Always access through HHDM offset for physical addresses

### Adding Shell Commands

1. Edit `kernel/src/shell/commands.rs`:
   - Add variant to `Command<'a>` enum
   - Implement `cmd_*()` function
   - Add match arm in `execute()`

2. Edit `kernel/src/shell/parser.rs`:
   - Add match arm in `parse()` function
   - Use `&'a str` slices to avoid allocations

3. Rebuild: `make iso && make run`

### Common Pitfalls

1. **Don't use heap allocations in shell** - Use stack buffers and string slices
2. **Don't forget volatile operations** - Compiler will optimize away MMIO reads/writes otherwise
3. **Check linker script** - Higher-half kernel requires proper virtual addresses
4. **Verify target** - Ensure building for `x86_64-unknown-none.json`, not default target
5. **Stack size** - Large arrays can overflow kernel stack (default 16KB per core)

## Testing Strategy

### Unit Tests
- Located in `shared/` crate (hardware-agnostic code)
- Run on host ARM64: `make test-host`
- Use `#[cfg(test)]` modules

### Integration Tests
- Kernel tests run in QEMU (x86_64)
- Use `#[test_case]` attribute with custom test framework
- Currently minimal due to no-std environment

### Manual Testing
1. Build and run: `make run`
2. Test shell commands interactively
3. Monitor serial output for debug messages
4. Verify no exceptions/panics in serial log

## Debugging

### Serial Output
- Most informative debugging method
- Automatically shown with `make run` (uses `-serial stdio`)
- Add `serial_println!()` anywhere for debugging

### QEMU Monitor
- Access with Ctrl+A then C (in -nographic mode)
- Useful commands:
  - `info registers` - CPU state
  - `info mem` - Memory mappings
  - `info pic` - PIC state

### Common Issues

**Kernel doesn't boot**:
- Check `file target/x86_64-unknown-none/debug/kernel` shows x86-64 ELF
- Verify ISO created: `ls -lh os.iso`
- Rebuild clean: `make clean && make iso`

**General Protection Fault**:
- GDT/IDT not loaded properly
- Stack overflow (reduce local array sizes)
- Invalid memory access (check HHDM offset usage)

**Keyboard not working**:
- Ensure PIC initialized and remapped
- Verify interrupts enabled (`sti` instruction)
- Check IRQ1 handler registered in IDT

## Important Files

- **`.cargo/config.toml`** - Forces x86_64 target, enables build-std, sets linker
- **`x86_64-unknown-none.json`** - Custom target spec with kernel code model
- **`kernel/linker.ld`** - Higher-half linker script (critical for virtual addressing)
- **`limine.conf`** - Bootloader configuration (protocol, kernel path)
- **`Makefile`** - Build orchestration (handles cross-compilation complexity)

## Limine Bootloader Protocol

The kernel uses Limine protocol v8.x requests:

- **HHDM Request** - Maps all physical memory to higher-half (`hhdm_offset`)
- **Memory Map Request** - Provides usable RAM regions
- **Kernel Address Request** - Reports kernel load location
- **Framebuffer Request** - For future graphics support (unused)

Requests are defined in `kernel/src/limine.rs` using static variables with special sections.
