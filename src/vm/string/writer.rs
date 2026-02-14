use std::io::{self, Write};
use std::ptr::NonNull;

use crate::vm::Value;
use crate::vm::gc::GarbageCollector;
use crate::vm::string::{OwnedStringHeader, StringHeader, StringRef};

pub struct StringBuffer {
    storage: NonNull<OwnedStringHeader>,
    len: usize,
}

pub struct StringWriter<'a> {
    buffer: &'a mut StringBuffer,
    gc: &'a mut GarbageCollector,
}

impl StringBuffer {
    pub fn new(gc: &mut GarbageCollector) -> Self {
        let storage = StringRef::new_zeroed(16, gc);
        debug_assert!(!storage.header().is_borrowed);
        let storage = storage.0.cast::<OwnedStringHeader>();
        Self { storage, len: 0 }
    }

    fn capacity(&self) -> usize {
        unsafe { self.storage.as_ref() }.len
    }

    fn really_realloc(&mut self, new_capacity: usize, gc: &mut GarbageCollector) {
        let new_capacity = new_capacity.max((self.len + 1).next_power_of_two());
        assert!(new_capacity < isize::MAX as usize, "string is too long");
        let new_string = StringRef::new_zeroed(new_capacity, gc);
        debug_assert!(!new_string.header().is_borrowed);
        let new_storage = new_string.0.cast::<OwnedStringHeader>();
        let old_data = OwnedStringHeader::content_ptr(self.storage);
        let new_data = OwnedStringHeader::content_ptr(new_storage);
        unsafe {
            new_data.copy_from_nonoverlapping(old_data, self.len);
        }
        self.storage = new_storage;
    }

    fn realloc(&mut self, new_capacity: usize, gc: &mut GarbageCollector) {
        if self.capacity() >= new_capacity {
            return;
        }
        self.really_realloc(new_capacity, gc);
    }

    fn storage_string(&self) -> StringRef {
        StringRef(self.storage.cast::<StringHeader>())
    }

    pub fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(Value::from_string_ref(self.storage_string()));
    }

    pub fn writer<'a>(&'a mut self, gc: &'a mut GarbageCollector) -> StringWriter<'a> {
        StringWriter { buffer: self, gc }
    }

    pub fn to_string(&self, gc: &mut GarbageCollector) -> StringRef {
        self.storage_string()
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
            OwnedStringHeader::content_ptr(self.buffer.storage)
                .add(self.buffer.len)
                .copy_from(NonNull::from(buf).cast::<u8>(), buf.len());
        }
        self.buffer.len = new_len;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
