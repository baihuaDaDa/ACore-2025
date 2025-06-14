ACore-2025
===

## Overview

ACore-2025 is a simple 64-bit RISC-V operating system kernel written in Rust, designed for educational purposes (final lab for CS2952).
It aims to provide a basic understanding of operating system concepts such as process management, memory management, and inter-process communication.

## TODO

- Bootloader
  - [x] Initialization
  - [x] Entering S mode for the kernel
- Allocator
  - [x] Buddy allocator
  - [x] Frame allocator (or any fine-grained allocator for any size of memory)
  - [ ] SLAB (Optional)
- Page table
  - [x] For kernel
  - [x] For each user process
- Console
  - [x] Read
  - [x] Write
- Message & data transfer
  - [x] User -> Kernel
  - [x] Kernel -> User
  - [x] Kernel -> Kernel
  - [x] User -> User
- Process
  - Process loading
    - [x] ELF parsing
    - [x] Sections loading (ref to page table)
  - Syscall
    - [x] Kick off a new process (Something like fork and exec)
    - [x] Wait for child processes (Something like wait)
    - [x] Exit from a process (Something like exit)
  - Process manager
    - [x] Process creation
    - [x] Process termination
  - Scheduler
    - [x] Context switch
    - [x] Scheduling mechanism (must be time sharing)
      - [ ] Advanced scheduling mechanism (Optional)
    - [x] Timer interrupt
    - [ ] IPI (Optional)
  - IPC
    - [x] Pipe
- Synchronization primitives
  - [x] Mutex
  - [ ] Conditional variables (Optional)
- File system (Optional)
  - [ ] File/directory creation/deletion
  - [ ] File/directory renaming
  - [x] File read
  - [x] File write
  - [ ] File/directory moving
  - [ ] (optional) access control, atime/mtime/…
- [ ] Multicore (Optional)
- [ ] Driver (Optional)

