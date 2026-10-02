pub mod writer;

use std::alloc::{self, Layout};
use std::fmt;
use std::fmt::Debug;
use std::hint::assert_unchecked;
use std::ptr::NonNull;

use crate::vm::Value;
use crate::vm::gc::{GarbageCollector, GcInfo};

#[repr(C, align(8))]
pub struct StringHeader {
    gc_info: GcInfo,
    bytes: NonNull<[u8]>,
    _marker: [StringRef; 0],
}

#[derive(Clone, Copy)]
pub struct StringRef(pub NonNull<StringHeader>);

impl StringRef {
    fn owned_layout(len: usize) -> Layout {
        let (layout, offset) = Layout::new::<StringHeader>()
            .extend(Layout::array::<u8>(len).unwrap())
            .unwrap();
        debug_assert_eq!(offset, size_of::<StringHeader>());
        layout
    }

    const BORROWED_LAYOUT: Layout = {
        let Ok((layout, offset)) = Layout::new::<StringHeader>().extend(Layout::new::<StringRef>())
        else {
            panic!("invalid layout");
        };
        assert!(offset == size_of::<StringHeader>());
        layout
    };

    fn header(&self) -> &StringHeader {
        unsafe { self.0.as_ref() }
    }

    pub fn bytes_non_null(self) -> NonNull<[u8]> {
        self.header().bytes
    }

    fn bytes_start(self) -> NonNull<u8> {
        self.bytes_non_null().cast::<u8>()
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe { self.bytes_non_null().as_ref() }
    }

    fn payload_ptr(self) -> NonNull<u8> {
        unsafe { self.0.add(1).cast::<u8>() }
    }

    fn is_owned(self) -> bool {
        self.bytes_start() == self.payload_ptr()
    }

    // Allocate memory for an owned string.
    unsafe fn allocate(len: usize, gc: &mut GarbageCollector) -> Self {
        let layout = Self::owned_layout(len);
        let this = Self(
            NonNull::new(unsafe { alloc::alloc(layout) })
                .unwrap()
                .cast::<StringHeader>(),
        );
        unsafe {
            this.0.write(StringHeader {
                gc_info: GcInfo::default(),
                bytes: NonNull::slice_from_raw_parts(this.payload_ptr(), len),
                _marker: [],
            });
        }
        gc.start_tracking(Value::from(this), layout.size());
        this
    }

    pub fn new(bytes: &[u8], gc: &mut GarbageCollector) -> Self {
        let this = unsafe { Self::allocate(bytes.len(), gc) };
        unsafe {
            assert_unchecked(this.is_owned());
            this.bytes_start()
                .copy_from_nonoverlapping(NonNull::from(bytes).cast::<u8>(), bytes.len());
        }
        this
    }

    pub fn new_zeroed(len: usize, gc: &mut GarbageCollector) -> Self {
        let this = unsafe { Self::allocate(len, gc) };
        unsafe {
            assert_unchecked(this.is_owned());
            this.bytes_start().write_bytes(0, len);
        }
        this
    }

    unsafe fn new_borrowed(
        owner: StringRef,
        bytes: NonNull<[u8]>,
        gc: &mut GarbageCollector,
    ) -> Self {
        let this = Self(
            NonNull::new(unsafe { alloc::alloc(Self::BORROWED_LAYOUT) })
                .unwrap()
                .cast::<StringHeader>(),
        );
        unsafe {
            this.0.write(StringHeader {
                gc_info: GcInfo::default(),
                bytes,
                _marker: [],
            });
            this.payload_ptr().cast::<StringRef>().write(owner);
        }
        gc.start_tracking(Value::from(this), Self::BORROWED_LAYOUT.size());
        this
    }

    pub fn layout(self) -> Layout {
        if self.is_owned() {
            Self::owned_layout(self.bytes_non_null().len())
        } else {
            Self::BORROWED_LAYOUT
        }
    }

    pub fn gc_mark(self) {
        let header = self.header();
        if header.gc_info.mark() || self.is_owned() {
            return;
        }
        let owner = *unsafe { self.payload_ptr().cast::<StringRef>().as_ref() };
        owner.gc_mark();
    }

    pub fn gc_sweep(self) -> bool {
        if self.header().gc_info.unmark() {
            return true;
        }
        unsafe {
            alloc::dealloc(self.0.as_ptr().cast::<u8>(), self.layout());
        }
        false
    }

    fn owner(self) -> Self {
        if self.is_owned() {
            self
        } else {
            *unsafe { self.payload_ptr().cast::<StringRef>().as_ref() }
        }
    }

    // Bytes must be a slice of self.bytes().
    pub unsafe fn slice_raw(self, bytes: NonNull<[u8]>, gc: &mut GarbageCollector) -> StringRef {
        let owner = self.owner();
        debug_assert!(owner.is_owned());
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
