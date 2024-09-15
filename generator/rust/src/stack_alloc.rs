use core::alloc::Layout;

struct StackAllocator {
    stack_ptr: usize,
}

static mut STACK_ALLOCATOR: StackAllocator = StackAllocator { stack_ptr: 0 };

impl StackAllocator {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let ptr = align_up(self.stack_ptr, align);
        self.stack_ptr = ptr + size;
        ptr as *mut u8
    }

    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // 在栈分配器中,我们不需要显式地释放内存
    }
}

#[no_mangle]
pub extern "C" fn stackAlloc(size: usize) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align(size, 1).unwrap();
        STACK_ALLOCATOR.alloc(layout)
    }
}

#[no_mangle]
pub extern "C" fn stackSave() -> usize {
    unsafe { STACK_ALLOCATOR.stack_ptr }
}

#[no_mangle]
pub extern "C" fn stackRestore(stack_ptr: usize) {
    unsafe {
        STACK_ALLOCATOR.stack_ptr = stack_ptr;
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
