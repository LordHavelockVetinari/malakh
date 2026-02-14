pub mod writer;

use std::alloc::{self, Layout};
use std::fmt::Debug;
use std::ptr::NonNull;
use std::{fmt, slice};

use crate::vm::Value;
use crate::vm::gc::{GarbageCollector, GcInfo};

#[repr(C, align(8))]
pub struct StringHeader {
    gc_info: GcInfo,
    is_borrowed: bool,
}

#[repr(C, align(8))]
pub struct BorrowedStringHeader {
    gc_info: GcInfo,
    is_borrowed: bool,
    owner: StringRef,
    bytes: NonNull<[u8]>,
}

#[repr(C, align(8))]
pub struct OwnedStringHeader {
    gc_info: GcInfo,
    is_borrowed: bool,
    len: usize,
}

#[derive(Clone)]
pub struct StringRef(pub NonNull<StringHeader>);

impl OwnedStringHeader {
    fn content_ptr(this: NonNull<Self>) -> NonNull<u8> {
        unsafe { this.add(1) }.cast::<u8>()
    }
}

impl StringRef {
    fn owned_layout(len: usize) -> Layout {
        let (layout, offset) = Layout::new::<OwnedStringHeader>()
            .extend(Layout::array::<u8>(len).unwrap())
            .unwrap();
        debug_assert_eq!(offset, size_of::<OwnedStringHeader>());
        layout
    }

    fn borrowed_layout() -> Layout {
        Layout::new::<BorrowedStringHeader>()
    }

    // Allocate memory for an owned string.
    unsafe fn allocate(len: usize, gc: &mut GarbageCollector) -> Self {
        let layout = Self::owned_layout(len);
        let this = Self(
            NonNull::new(unsafe { alloc::alloc(layout) })
                .unwrap()
                .cast::<StringHeader>(),
        );
        let header = this.0.cast::<OwnedStringHeader>();
        unsafe {
            header.write(OwnedStringHeader {
                gc_info: GcInfo::default(),
                is_borrowed: false,
                len,
            });
        }
        gc.start_tracking(Value::from_string_ref(this.clone()), layout.size());
        this
    }

    pub fn new(bytes: &[u8], gc: &mut GarbageCollector) -> Self {
        let this = unsafe { Self::allocate(bytes.len(), gc) };
        debug_assert!(!this.header().is_borrowed);
        let header = this.0.cast::<OwnedStringHeader>();
        let content = OwnedStringHeader::content_ptr(header);
        unsafe {
            content.copy_from_nonoverlapping(NonNull::from(bytes).cast::<u8>(), bytes.len());
        }
        this
    }

    pub fn new_zeroed(len: usize, gc: &mut GarbageCollector) -> Self {
        let this = unsafe { Self::allocate(len, gc) };
        debug_assert!(!this.header().is_borrowed);
        let header = this.0.cast::<OwnedStringHeader>();
        let content = OwnedStringHeader::content_ptr(header);
        unsafe {
            content.write_bytes(0, len);
        }
        this
    }

    unsafe fn new_borrowed(
        owner: StringRef,
        bytes: NonNull<[u8]>,
        gc: &mut GarbageCollector,
    ) -> Self {
        let layout = Self::borrowed_layout();
        let this = Self(
            NonNull::new(unsafe { alloc::alloc(layout) })
                .unwrap()
                .cast::<StringHeader>(),
        );
        let header = this.0.cast::<BorrowedStringHeader>();
        unsafe {
            header.write(BorrowedStringHeader {
                gc_info: GcInfo::default(),
                is_borrowed: true,
                owner,
                bytes,
            });
        }
        gc.start_tracking(Value::from_string_ref(this.clone()), layout.size());
        this
    }

    fn header(&self) -> &StringHeader {
        unsafe { self.0.as_ref() }
    }

    pub fn bytes(&self) -> &[u8] {
        if self.header().is_borrowed {
            unsafe {
                let header = self.0.cast::<BorrowedStringHeader>().as_ref();
                header.bytes.as_ref()
            }
        } else {
            let header = self.0.cast::<OwnedStringHeader>();
            let content = OwnedStringHeader::content_ptr(header);
            unsafe { slice::from_raw_parts(content.as_ptr(), header.as_ref().len) }
        }
    }

    fn layout(&self) -> Layout {
        if self.header().is_borrowed {
            Self::borrowed_layout()
        } else {
            let header = unsafe { self.0.cast::<OwnedStringHeader>().as_ref() };
            Self::owned_layout(header.len)
        }
    }

    pub fn gc_mark(&self) {
        let header = self.header();
        if header.gc_info.mark() || !header.is_borrowed {
            return;
        }
        let header = unsafe { self.0.cast::<BorrowedStringHeader>().as_ref() };
        header.owner.gc_mark();
    }

    pub fn gc_sweep(&mut self) -> bool {
        if self.header().gc_info.unmark() {
            return true;
        }
        unsafe {
            alloc::dealloc(self.0.as_ptr().cast::<u8>(), self.layout());
        }
        false
    }

    fn owner(&self) -> StringRef {
        if self.header().is_borrowed {
            unsafe { self.0.cast::<BorrowedStringHeader>().as_ref().owner.clone() }
        } else {
            self.clone()
        }
    }

    // Bytes must be a slice of self.bytes().
    pub unsafe fn slice_raw(&self, bytes: NonNull<[u8]>, gc: &mut GarbageCollector) -> StringRef {
        let owner = self.owner();
        debug_assert!(!owner.header().is_borrowed);
        unsafe { Self::new_borrowed(owner, bytes, gc) }
    }

    pub fn slice(&self, offset: usize, len: usize, gc: &mut GarbageCollector) -> Option<StringRef> {
        let bytes = NonNull::from(self.bytes().get(offset..offset + len)?);
        Some(unsafe { self.slice_raw(bytes, gc) })
    }
}

impl Debug for StringRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.bytes()))
    }
}
