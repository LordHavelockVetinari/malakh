use std::ptr::NonNull;

use malachite::Integer;

use crate::vm::Value;
use crate::vm::gc::{GarbageCollector, GcInfo};

#[repr(align(8))]
pub struct BigIntInner {
    gc_info: GcInfo,
    value: Integer,
}

#[derive(Clone, Debug)]
pub struct BigIntRef(pub NonNull<BigIntInner>);

impl BigIntRef {
    // Doesn't check that the value is large.
    // (If it's small, it shouldn't be a BigIntRef.)
    pub fn new_unchecked(value: Integer, gc: &mut GarbageCollector) -> Self {
        let approx_size =
            value.unsigned_abs_ref().limb_count() as usize * size_of::<malachite::platform::Limb>();
        let ptr = Box::into_raw(Box::new(BigIntInner {
            gc_info: GcInfo::default(),
            value,
        }));
        let this = BigIntRef(NonNull::new(ptr).unwrap());
        gc.start_tracking(Value::from_big_int_ref(this.clone()), approx_size);
        this
    }

    pub fn value(self) -> *const Integer {
        unsafe { &raw const (*self.0.as_ptr()).value }
    }

    pub fn gc_mark(&self) {
        unsafe { self.0.as_ref() }.gc_info.mark();
    }

    pub fn gc_sweep(&mut self) -> bool {
        if unsafe { self.0.as_ref() }.gc_info.unmark() {
            return true;
        }
        unsafe {
            drop(Box::<BigIntInner>::from_raw(self.0.as_ptr()));
        }
        false
    }
}
