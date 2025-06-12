use crate::sync::UPSafeCell;
use crate::task::{TaskControlBlock, suspend_current_and_run_next, current_task, block_current_and_run_next, wakeup_task};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::RefMut;

pub trait Mutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
}

pub struct MutexSpin {
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    pub fn locked_exclusive_access(&self) -> RefMut<'_, bool> {
        self.locked.exclusive_access()
    }

    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    fn lock(&self) {
        loop {
            let mut locked = self.locked_exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        let mut locked = self.locked_exclusive_access();
        *locked = false;
    }
}

pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, MutexBlockingInner> {
        self.inner.exclusive_access()
    }

    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    fn lock(&self) {
        let mut mutex_inner = self.inner_exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner_exclusive_access();
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            // Wake up the first task in the wait queue
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
