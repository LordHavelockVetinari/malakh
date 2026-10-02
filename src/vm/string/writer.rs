use std::hint::assert_unchecked;
use std::io::{self, Write};
use std::ptr::NonNull;

use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;

pub struct StringBuffer {
    storage: StringRef,
    len: usize,
}

pub struct StringWriter<'a> {
    buffer: &'a mut StringBuffer,
    gc: &'a mut GarbageCollector,
}

impl StringBuffer {
    pub fn new(gc: &mut GarbageCollector) -> Self {
        let storage = StringRef::new_zeroed(16, gc);
        Self { storage, len: 0 }
    }

    fn storage_bytes_non_null(&self) -> NonNull<[u8]> {
        unsafe {
            assert_unchecked(self.storage.is_owned());
        }
        self.storage.bytes_non_null()
    }

    fn storage_bytes(&self) -> &[u8] {
        unsafe { self.storage_bytes_non_null().as_ref() }
    }

    fn capacity(&self) -> usize {
        self.storage_bytes_non_null().len()
    }

    fn really_realloc(&mut self, new_capacity: usize, gc: &mut GarbageCollector) {
        let new_capacity = new_capacity.max((self.len + 1).next_power_of_two());
        assert!(new_capacity < isize::MAX as usize, "string is too long");
        let new_string = StringRef::new_zeroed(new_capacity, gc);
        let old_data = self.storage_bytes();
        let new_data = unsafe { new_string.bytes_non_null().as_mut() };
        new_data[..self.len].copy_from_slice(&old_data[..self.len]);
        self.storage = new_string;
    }

    fn realloc(&mut self, new_capacity: usize, gc: &mut GarbageCollector) {
        if self.capacity() >= new_capacity {
            return;
        }
        self.really_realloc(new_capacity, gc);
    }

    pub fn gc_mark_content(&self, _gc: &mut GarbageCollector) {
        self.storage.gc_mark();
    }

    pub fn writer<'a>(&'a mut self, gc: &'a mut GarbageCollector) -> StringWriter<'a> {
        StringWriter { buffer: self, gc }
    }

    pub fn to_string(&self, gc: &mut GarbageCollector) -> StringRef {
        self.storage
            .slice(0, self.len, gc)
            .expect("failed to slice string")
    }
}

impl Write for StringWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let new_len = self.buffer.len.checked_add(buf.len()).unwrap();
        self.buffer.realloc(new_len, self.gc);
        debug_assert!(self.buffer.capacity() >= new_len);
        unsafe {
            let dest = self
                .buffer
                .storage_bytes_non_null()
                .cast::<u8>()
                .add(self.buffer.len);
            dest.copy_from(NonNull::from(buf).cast::<u8>(), buf.len());
        }
        self.buffer.len = new_len;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
