# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := target/$(TARGET)/$(MODE)/os.bin

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

build: kernel $(KERNEL_BIN)

kernel:
	@cd ../user && make build
	@echo Paltform: $(BOARD)
	@cargo build $(MODE_ARG)

$(KERNEL_BIN):
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

QEMU_ARGS := -machine virt \
             			 -nographic \
             			 -bios $(BOOTLOADER) \
             			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
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