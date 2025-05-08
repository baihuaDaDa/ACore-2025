#[derive(Clone, Copy)]
pub struct LinkedList {
    head: *mut usize,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    pub const fn new() -> Self {
        Self { head: core::ptr::null_mut() }
    }
    
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }
    
    pub unsafe fn push(&mut self, item: *mut usize) {
        unsafe { *item = self.head as usize };
        self.head = item;
    }
    
    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                let item = self.head;
                self.head = unsafe { *item as *mut usize };
                Some(item)
            }
        }
    }
    
    pub fn iter_mut(&mut self) -> LinkedListIteratorMut {
        LinkedListIteratorMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            list: self,
        }
    }
}

pub struct LinkedNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl LinkedNode {
    pub fn pop(self) -> *mut usize {
        unsafe {
            *self.prev = *self.curr;
        }
        self.curr
    }
    
    pub fn value(&self) -> *mut usize {
        self.curr
    }
}

pub struct LinkedListIteratorMut<'a> {
    list: &'a mut LinkedList,
    prev: *mut usize,
    curr: *mut usize,
}

impl<'a> Iterator for LinkedListIteratorMut<'a> {
    type Item = LinkedNode;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let ret = Self::Item {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            unsafe { self.curr = *self.curr as *mut usize };
            Some(ret)
        }
    }
}