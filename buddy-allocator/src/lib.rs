#![no_std]

mod linked_list;
mod math;

use spin::{Mutex, MutexGuard};
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use linked_list::LinkedList;
use math::prev_power_of_two;

const BLOCK_LEVEL: usize = 32;
const UNIT_SIZE: usize = size_of::<usize>();

pub struct BuddyAllocator {
    free_list: [LinkedList; BLOCK_LEVEL],
    total: usize,
    user: usize,
    allocated: usize,
}

impl BuddyAllocator {
    pub const fn empty() -> Self {
        Self {
            free_list: [LinkedList::new(); BLOCK_LEVEL],
            total: 0,
            user: 0,
            allocated: 0,
        }
    }

    // add [start, end) space to heap (aligned)
    unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        // align
        start = (start + UNIT_SIZE - 1) & !(UNIT_SIZE - 1);
        end = end & !(UNIT_SIZE - 1);
        assert!(start <= end);
        // split space
        let mut current_start = start;
        let mut total = 0;
        while current_start < end {
            let low_bit = current_start & (!current_start + 1);
            let size = min(low_bit, prev_power_of_two(end - current_start));
            unsafe { self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize); }
            current_start += size;
            total += size;
        }
        assert_eq!(end - start, total);
        self.total += total;
    }
    
    pub unsafe fn init(&mut self, start: usize, heap_size: usize) {
        unsafe { self.add_to_heap(start, start + heap_size); }
    }
    
    fn split(&mut self, from: usize, to: usize) {
        for i in (to + 1 ..=from).rev() {
            if let Some(block) = self.free_list[i].pop() {
                unsafe { 
                    self.free_list[i - 1].push(block);
                    self.free_list[i - 1].push((block as usize + (1 << (i - 1))) as *mut usize);
                }
            } else {
                panic!("[kernel] (Buddy Allocator) Failed to split block!");
            }
        }
    }
    
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // align the size
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), UNIT_SIZE)
        );
        let level = size.trailing_zeros() as usize; // log
        for i in level..BLOCK_LEVEL {
            if !self.free_list[i].is_empty() {
                self.split(i, level);
                self.user += layout.size();
                self.allocated += size;
                return self.free_list[level].pop().unwrap() as *mut u8;
            }
        }
        panic!("[kernel] (Buddy Allocator) Heap memory run out!");
    }
    
    fn merge(&mut self, from: usize, ptr: *mut u8) {
        let mut current_ptr = ptr as usize;
        let mut current_level = from;
        while current_level < BLOCK_LEVEL {
            let buddy = current_ptr ^ (1 << current_level);
            let mut flag = false;
            for block in self.free_list[current_level].iter_mut() {
                if block.value() as usize == buddy {
                    block.pop();
                    flag = true;
                    break;
                }
            }
            if flag {
                self.free_list[current_level].pop();
                current_ptr = min(current_ptr, buddy);
                current_level += 1;
                unsafe { self.free_list[current_level].push(current_ptr as *mut usize) };
            } else {
                break;
            }
        }
    }
    
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), UNIT_SIZE)
        );
        let level = size.trailing_zeros() as usize;
        unsafe { self.free_list[level].push(ptr as *mut usize); }
        self.merge(level, ptr);
        self.user -= layout.size();
        self.allocated -= size;
    }
}

pub struct LockedBuddyAllocator(Mutex<BuddyAllocator>);

impl LockedBuddyAllocator {
    pub const fn empty() -> Self {
        Self(Mutex::new(BuddyAllocator::empty()))
    }

    pub fn lock(&self) -> MutexGuard<BuddyAllocator> {
        self.0.lock()
    }
}

unsafe impl GlobalAlloc for LockedBuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(ptr, layout)
    }
}