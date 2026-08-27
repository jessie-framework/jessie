pub struct Arena {
    mem: Box<[core::mem::MaybeUninit<u8>]>,
    bump: std::cell::Cell<u32>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            mem: Box::new_uninit_slice(u32::MAX as usize),
            bump: std::cell::Cell::new(0),
        }
    }

    pub fn alloc<I: ArenaAllocable>(&self, item: I) -> &I {
        unsafe { item.alloc(self) }
    }

    pub fn alloc_str(&self, input: &str) -> &str {
        unsafe {
            let ptr: *mut u8 = self
                .mem
                .as_ptr()
                .cast_mut()
                .byte_add(self.bump.get() as usize)
                .cast();
            let slice = core::slice::from_raw_parts_mut(ptr, input.len());
            for (idx, ch) in input.bytes().enumerate() {
                slice[idx] = ch;
            }
            self.bump.update(|x| x + input.len() as u32);
            core::str::from_utf8_unchecked(slice)
        }
    }

    pub fn alloc_slice<I: ArenaAllocable + Copy + 'static>(&self, input: &[I]) -> &[I] {
        unsafe {
            let ptr: *mut u8 = self
                .mem
                .as_ptr()
                .cast_mut()
                .byte_add(self.bump.get() as usize)
                .cast();
            self.bump
                .update(|x| x + ptr.align_offset(core::mem::align_of::<I>()) as u32);
            let ptr: *mut I = self.mem.as_ptr().cast_mut().cast();
            for (idx, val) in input.iter().enumerate() {
                ptr.add(idx).write(*val);
            }
            self.bump
                .update(|x| x + core::mem::size_of_val(input) as u32);
            core::slice::from_raw_parts(ptr, input.len())
        }
    }
}

/// Allocate data in the arena.
/// # Safety
/// By default this function just copies over the bytes of `self` over into the area , which may cause dangling references
pub unsafe trait ArenaAllocable: Sized {
    /// # Safety
    /// An incorrect implementation may cause dangling references
    unsafe fn alloc(self, arena: &Arena) -> &Self;
}

unsafe impl<T: 'static + Copy> ArenaAllocable for T {
    /// # Safety
    /// By default this function just copies over the bytes of `self` over into the area , which may cause dangling references
    unsafe fn alloc(self, arena: &Arena) -> &Self {
        unsafe {
            let offset = arena
                .mem
                .as_ptr()
                .byte_add(arena.bump.get() as usize)
                .align_offset(core::mem::align_of::<Self>());

            arena.bump.update(|x| x + offset as u32);

            let ptr = arena
                .mem
                .as_ptr()
                .byte_add(arena.bump.get() as usize)
                .cast_mut()
                .cast();

            arena
                .bump
                .update(|x| x + core::mem::size_of::<Self>() as u32);

            *ptr = self;

            ptr.as_ref_unchecked()
        }
    }
}

#[test]
fn test_arena() {
    let arena = Arena::new();
    let one = arena.alloc(1);
    let one_two_three = arena.alloc_slice(&[1, 2, 3]);
    let two = arena.alloc(2);
    let hello = arena.alloc_str("hello");
    let three = arena.alloc(3);
    assert_eq!(*one, 1);
    assert_eq!(*two, 2);
    assert_eq!(*three, 3);
    assert_eq!(hello, "hello");
    assert_eq!(one_two_three, &[1, 2, 3]);
}
