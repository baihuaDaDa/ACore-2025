# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := target/$(TARGET)/$(MODE)/os.bin
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin

TEST ?=

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build: $(KERNEL_BIN) fs-img

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Paltform: $(BOARD)
	@cargo build $(MODE_ARG)

apps:
	@cd ../user && make build TEST=$(TEST) MODE=$(MODE) TARGET=$(TARGET) OBJCOPY=$(OBJCOPY) OBJDUMP=$(OBJDUMP)

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
             			 -bios $(BOOTLOADER) \
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