# Makefile for wflos - Rust Microkernel OS
# Handles M1 cross-compilation: kernel (x86_64) vs Limine utility (ARM64)

KERNEL_ARCH := x86_64-unknown-none
HOST_ARCH := $(shell uname -m)
KERNEL_BINARY := target/$(KERNEL_ARCH)/debug/kernel
ISO_IMAGE := os.iso

.PHONY: all kernel limine-utility iso run clean test test-host test-integration

all: iso

# Build kernel for x86_64 target using rust-lld
kernel:
	@echo "Building kernel for x86_64..."
	cargo +nightly build --target $(KERNEL_ARCH).json
	@echo "Verifying kernel is ELF x86-64..."
	@file $(KERNEL_BINARY)

# Clone and build Limine utility for host ARM64 architecture
limine-utility:
	@if [ ! -d "build_limine" ]; then \
		echo "Cloning Limine bootloader (v8.x-binary)..."; \
		git clone --branch=v8.x-binary --depth=1 \
			https://github.com/limine-bootloader/limine.git build_limine; \
	fi
	@echo "Building Limine host utility for $(HOST_ARCH)..."
	$(MAKE) -C build_limine
	@echo "Verifying Limine utility architecture..."
	@file build_limine/limine

# Create bootable ISO image
iso: kernel limine-utility
	@echo "Creating bootable ISO..."
	@rm -rf iso_root
	@mkdir -p iso_root/boot
	@mkdir -p iso_root/boot/limine
	@mkdir -p iso_root/EFI/BOOT
	@cp $(KERNEL_BINARY) iso_root/boot/kernel
	@cp limine.conf iso_root/boot/limine/limine.conf
	@cp build_limine/limine-bios.sys iso_root/boot/limine/
	@cp build_limine/limine-bios-cd.bin iso_root/boot/limine/
	@cp build_limine/limine-uefi-cd.bin iso_root/boot/limine/
	@cp build_limine/BOOTX64.EFI iso_root/EFI/BOOT/
	@cp build_limine/BOOTIA32.EFI iso_root/EFI/BOOT/
	@xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(ISO_IMAGE) 2>/dev/null
	@./build_limine/limine bios-install $(ISO_IMAGE) 2>/dev/null
	@echo "ISO created: $(ISO_IMAGE)"

# Run kernel in QEMU (TCG emulation on M1)
run: iso
	@echo "Starting QEMU (x86_64 emulation on ARM64 may be slow)..."
	qemu-system-x86_64 -cdrom $(ISO_IMAGE) \
		-serial stdio \
		-no-reboot \
		-no-shutdown \
		-m 256M

# Run tests
test: test-host test-integration

test-host:
	@echo "Running host-based unit tests..."
	cargo test -p shared

test-integration:
	@echo "Running QEMU integration tests..."
	cargo +nightly test --target $(KERNEL_ARCH).json

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf iso_root $(ISO_IMAGE)
	@rm -rf build_limine

# Verify cross-compilation setup
verify:
	@echo "=== Cross-Compilation Verification ==="
	@echo "Host architecture: $(HOST_ARCH)"
	@echo "Kernel target: $(KERNEL_ARCH)"
	@echo ""
	@if [ -f "$(KERNEL_BINARY)" ]; then \
		echo "Kernel binary:"; \
		file $(KERNEL_BINARY); \
	else \
		echo "Kernel not built yet (run 'make kernel')"; \
	fi
	@echo ""
	@if [ -f "build_limine/limine" ]; then \
		echo "Limine utility:"; \
		file build_limine/limine; \
	else \
		echo "Limine not built yet (run 'make limine-utility')"; \
	fi
