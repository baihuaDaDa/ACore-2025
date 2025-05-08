# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := target/$(TARGET)/$(MODE)/os.bin
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin

TEST ?=
SBI ?=

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY & BIOS & LINKER
ifeq ($(SBI), rustsbi)
	KERNEL_ENTRY_PA := 0x80200000
	BIOS := $(BOOTLOADER)
	LINKER := src/linker-$(BOARD)-$(SBI).ld
else
	KERNEL_ENTRY_PA := 0x80000000
	BIOS := none
	LINKER := src/linker-$(BOARD)-mysbi.ld
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build: $(KERNEL_BIN) fs-img

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Paltform: $(BOARD)
	@cp $(LINKER) src/linker.ld
	@cargo build $(MODE_ARG)
	@rm src/linker.ld

apps:
	@cd ../user && make build TEST=$(TEST) MODE=$(MODE) TARGET=$(TARGET)

fs-img: apps
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin -t ../user/target/$(TARGET)/$(MODE)/

clean:
	@cargo clean

clean-user:
	@cd ../user && make clean

clean-all: clean clean-user

QEMU_ARGS := -machine virt \
             			 -nographic \
             			 -bios $(BIOS) \
             			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
             			 -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
             			 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMU_NAME := qemu-system-riscv64

run: run-inner

run-inner: build
	@$(QEMU_NAME) $(QEMU_ARGS)

debug: build
	@tmux new-session -d \
            "$(QEMU_NAME) $(QEMU_ARGS) -s -S" && \
            tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
            tmux -2 attach-session -d

gdbserver: build
	@$(QEMU_NAME) $(QEMU_ARGS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

.PHONY: build kernel clean run run-inner debug gdbserver gdbclient