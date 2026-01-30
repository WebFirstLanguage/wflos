# wflos Usage Guide

## Quick Start

### Build and Run

```bash
# Build the kernel and create bootable ISO
make iso

# Run in QEMU (graphical mode with VGA)
qemu-system-x86_64 -cdrom os.iso -m 256M

# Run in QEMU with serial output
qemu-system-x86_64 -cdrom os.iso -m 256M -serial stdio
```

### First Boot

1. QEMU window opens
2. Boot sequence displays:
   ```
   wflos - Rust Microkernel OS
   Version 0.4.0 (Phase 4: Command-Line Interface)
   ```
3. Initialization messages appear
4. Shell banner displays:
   ```
   === wflos Shell ===
   Type 'help' for available commands
   ```
5. Prompt appears: `wflos> `
6. You can now type commands!

---

## Shell Commands

### `help` - Show Command List

```
wflos> help
Available commands:
  help      - Show this help message
  clear     - Clear the screen
  echo TEXT - Print text to screen
  version   - Show kernel version
  meminfo   - Display memory information
  halt      - Halt the system
```

### `version` - Kernel Information

```
wflos> version
wflos - Rust Microkernel OS
Version 0.4.0 (Phase 4: Command-Line Interface)
Built with Rust on Apple Silicon M1 for x86_64

Features:
  - Cross-compilation (ARM64 -> x86_64)
  - Limine bootloader protocol
  - VGA text mode driver
  - Serial port debugging
  - GDT and IDT configured
  - Physical frame allocator
  - PS/2 keyboard input
  - Interactive shell
```

### `echo` - Print Text

```
wflos> echo Hello, World!
Hello, World!

wflos> echo Welcome to wflos!
Welcome to wflos!
```

### `meminfo` - Memory Statistics

```
wflos> meminfo
Memory Information:
  Total frames: 64219 (250 KB)
  Used frames:  0 (0 KB)
  Free frames:  64219 (250 KB)

Frame size: 4 KB
```

### `clear` - Clear Screen

```
wflos> clear
```
(Screen clears, new prompt appears at top)

### `halt` - Stop System

```
wflos> halt
Halting system...
You can close QEMU or press Ctrl+A then X to exit.
```
(System enters halt loop, CPU sleeps)

---

## Keyboard Controls

### Typing
- **Letters**: a-z (lowercase only)
- **Numbers**: 0-9
- **Punctuation**: Most standard symbols
- **Space**: Space bar

### Editing
- **Backspace**: Delete previous character
  - Visual feedback: character erased on screen
  - Stops at beginning of line
- **Enter**: Execute command
  - Moves to new line
  - Parses and runs command
  - Shows new prompt
- **ESC**: Clear entire line
  - Erases all typed characters
  - Cursor returns to start

### Special Keys
- **Tab**: Ignored (not implemented)
- **Arrow Keys**: Not supported yet
- **Ctrl Keys**: Not implemented

---

## QEMU Tips

### Running with Graphics

```bash
# Standard run
make run

# With more memory
qemu-system-x86_64 -cdrom os.iso -m 512M

# With serial output too
qemu-system-x86_64 -cdrom os.iso -m 256M -serial stdio
```

### Keyboard in QEMU

- Click in QEMU window to focus
- Type normally
- Ctrl+Alt+G to release mouse/keyboard from QEMU

### Exiting QEMU

- **Menu**: Machine → Quit
- **Keyboard**: Ctrl+A then X (in -nographic mode)
- **Shell**: Type `halt` command first (recommended)

### Debugging

```bash
# Show serial output
qemu-system-x86_64 -cdrom os.iso -m 256M -serial stdio

# Show interrupt info (for debugging)
qemu-system-x86_64 -cdrom os.iso -m 256M -d int -D debug.log

# Slow down for observation
qemu-system-x86_64 -cdrom os.iso -m 256M -icount 1
```

---

## Development Workflow

### Typical Development Cycle

1. **Edit Code**
   ```bash
   vim kernel/src/shell/commands.rs  # Add new command
   ```

2. **Build**
   ```bash
   make iso  # Rebuilds kernel and ISO
   ```

3. **Test**
   ```bash
   make run  # Launch in QEMU
   ```

4. **Debug**
   - Check serial output for debug messages
   - Test commands in shell
   - Verify behavior

5. **Iterate**

### Adding New Commands

1. Edit `kernel/src/shell/commands.rs`:
   ```rust
   // Add variant to Command enum
   pub enum Command<'a> {
       // ...existing...
       MyCommand,
   }

   // Add execution
   fn cmd_mycommand() {
       println!("My command output");
   }

   pub fn execute(cmd: Command) {
       match cmd {
           // ...existing...
           Command::MyCommand => cmd_mycommand(),
       }
   }
   ```

2. Edit `kernel/src/shell/parser.rs`:
   ```rust
   match cmd {
       // ...existing...
       "mycommand" => Ok(Command::MyCommand),
   }
   ```

3. Rebuild and test:
   ```bash
   make iso && make run
   ```

---

## Troubleshooting

### Kernel Doesn't Boot

**Symptom**: QEMU shows blank screen or boot error

**Checks**:
1. Verify kernel is ELF x86-64:
   ```bash
   file target/x86_64-unknown-none/debug/kernel
   ```
   Should show: `ELF 64-bit LSB executable, x86-64`

2. Check ISO was created:
   ```bash
   ls -lh os.iso
   ```

3. Rebuild from clean:
   ```bash
   make clean && make iso
   ```

### No Serial Output

**Symptom**: `-serial stdio` shows nothing

**Checks**:
1. Use `-nographic` instead:
   ```bash
   qemu-system-x86_64 -cdrom os.iso -m 256M -nographic
   ```

2. Check serial initialization in code

### Keyboard Not Working

**Symptom**: Can't type in shell

**Checks**:
1. Click in QEMU window to focus
2. Check if interrupts are enabled (serial log should show)
3. Verify PIC initialization succeeded

### General Protection Fault

**Symptom**: "EXCEPTION: General Protection Fault" on boot

**Common Causes**:
1. GDT/IDT not properly loaded
2. Stack overflow (large local arrays)
3. Invalid memory access

**Solution**: Check serial output for last successful step

---

## Example Sessions

### Session 1: Basic Exploration

```
wflos> help
Available commands:
  help      - Show this help message
  clear     - Clear the screen
  echo TEXT - Print text to screen
  version   - Show kernel version
  meminfo   - Display memory information
  halt      - Halt the system

wflos> version
wflos - Rust Microkernel OS
Version 0.4.0 (Phase 4: Command-Line Interface)
...

wflos> meminfo
Memory Information:
  Total frames: 64219 (250 KB)
  ...
```

### Session 2: Echo and Clear

```
wflos> echo Testing the shell
Testing the shell

wflos> echo Line 1
Line 1

wflos> echo Line 2
Line 2

wflos> clear
(screen clears)

wflos> echo Fresh start
Fresh start
```

### Session 3: Error Handling

```
wflos> unknown
Error: Unknown command. Type 'help' for available commands.

wflos> help
Available commands:
  ...

wflos> echo works now
works now
```

---

## Performance Notes

### Boot Time
- **QEMU on M1**: ~10 seconds (TCG emulation)
- **QEMU on x86**: ~2 seconds (native)
- **Real Hardware**: ~1 second (untested)

### Responsiveness
- **Keyboard**: Instant (interrupt-driven)
- **Commands**: Instant execution
- **VGA**: No perceptible lag

### Memory Usage
- **Kernel**: ~50 KB binary
- **Runtime**: < 1 KB RAM used
- **Available**: 250 MB+ RAM free

---

## Advanced Usage

### Serial Debugging

Monitor serial output while using shell:

```bash
qemu-system-x86_64 -cdrom os.iso -m 256M -serial stdio
```

Serial shows:
- Initialization steps
- Debug messages
- Error details
- System state

### QEMU Monitor

Access QEMU monitor (debugging):

```bash
# In -nographic mode: Ctrl+A then C
# Commands:
info registers  # Show CPU registers
info mem        # Show memory mappings
info pic        # Show PIC state
quit            # Exit QEMU
```

### Automation

Run commands automatically (future):

```bash
# Create command file
echo -e "version\nmeminfo\nhalt" > commands.txt

# Boot with commands (not implemented yet)
# Would need command file support in kernel
```

---

## Common Use Cases

### 1. Quick Test

```bash
make iso && make run
# Type 'help' then 'halt'
```

### 2. Memory Check

```bash
make run
wflos> meminfo
wflos> halt
```

### 3. Demo

```bash
make run
wflos> version
wflos> echo Welcome to wflos!
wflos> help
wflos> meminfo
wflos> clear
wflos> echo Demo complete
wflos> halt
```

### 4. Development Testing

```bash
make iso && make run
# Test new feature
wflos> mycommand
# Verify output
wflos> halt
```

---

## Tips & Tricks

### 1. Fast Rebuild
```bash
# Only kernel changed - use make iso (skips Limine rebuild)
make iso
```

### 2. Serial Debugging
Add `serial_println!()` anywhere in code for debugging:
```rust
serial_println!("Debug: variable = {}", value);
```

### 3. Quick Exit
Type `halt` to cleanly stop the kernel before closing QEMU

### 4. Screen Management
Use `clear` to declutter the screen during long sessions

### 5. Testing Commands
Test new commands with `echo` first to verify parsing works

---

## Support & Documentation

- **Architecture Guide**: `Docs/Building Rust Microkernel OS on Mac.md`
- **Phase Summaries**: `PHASE*_SUMMARY.md` files
- **MVP Overview**: `MVP_COMPLETE.md`
- **Source Code**: Well-commented inline documentation
- **Build System**: `Makefile` with comments

---

## Frequently Asked Questions

**Q: Can I run this on real hardware?**
A: Theoretically yes, but untested. You'd need to write the ISO to a USB drive.

**Q: Why lowercase only?**
A: Shift key support not implemented yet. Coming in future update.

**Q: Can I add my own commands?**
A: Yes! See "Adding New Commands" section above.

**Q: Why is QEMU slow on M1?**
A: TCG emulation (software emulation of x86_64 on ARM64). Native x86 hardware is much faster.

**Q: Is there tab completion?**
A: Not yet. Future feature.

**Q: Can I run multiple commands?**
A: Not yet. Each command runs individually.

**Q: What about copy/paste?**
A: QEMU supports paste. Copy not implemented in kernel.

---

**Happy kernel hacking! 🎉**
