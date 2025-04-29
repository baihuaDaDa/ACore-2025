ACore-2025
===

## Overview

## TODO

- Bootloader
  - [ ] Initialization
  - [ ] Entering S mode for the kernel
- Allocator
  - [ ] Buddy allocator
  - [x] Frame allocator (or any fine-grained allocator for any size of memory)
  - [ ] SLAB (Optional)
- Page table
  - [x] For kernel
  - [x] For each user process
- Console
  - [x] Read
  - [x] Write
- Message & data transfer
  - [ ] User -> Kernel
  - [ ] Kernel -> User
  - [ ] Kernel -> Kernel
  - [ ] User -> User
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
    - [ ] Pipe
- Synchronization primitives
  - [ ] Mutex
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

