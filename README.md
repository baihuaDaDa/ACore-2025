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
    - [ ] ELF parsing
    - [ ] Sections loading (ref to page table)
  - Syscall
    - [ ] Kick off a new process (Something like fork and exec)
    - [ ] Wait for child processes (Something like wait)
    - [ ] Exit from a process (Something like exit)
  - Process manager
    - [ ] Process creation
    - [ ] Process termination
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
  - [ ] File read
  - [ ] File write
  - [ ] File/directory moving
  - [ ] (optional) access control, atime/mtime/…
- [ ] Multicore (Optional)
- [ ] Driver (Optional)

