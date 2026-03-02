use std::ptr::NonNull;

use crate::vm::Value;
use crate::vm::gc::{GarbageCollector, GcInfo};

#[repr(C, align(8))]
pub struct FloatData {
    gc_info: GcInfo,
    value: f64,
}

#[derive(Clone, Copy)]
pub struct FloatRef(pub NonNull<FloatData>);

impl FloatRef {
    pub fn new(value: f64, gc: &mut GarbageCollector) -> Self {
        let this = FloatRef(
            NonNull::new(Box::into_raw(Box::new(FloatData {
                gc_info: GcInfo::default(),
                value,
            })))
            .unwrap(),
        );
        gc.start_tracking(Value::from(this), size_of::<FloatData>());
        this
    }

    pub fn value(&self) -> f64 {
        unsafe { self.0.as_ref().value }
    }

    pub fn gc_mark(&self) {
        unsafe { self.0.as_ref() }.gc_info.mark();
    }

    pub fn gc_sweep(&mut self) -> bool {
        if unsafe { self.0.as_ref() }.gc_info.unmark() {
            return true;
        }
        drop(unsafe { Box::<FloatData>::from_raw(self.0.as_ptr()) });
        false
    }
}
