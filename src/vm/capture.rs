use std::ptr::NonNull;

use crate::vm::Value;
use crate::vm::gc::{GarbageCollector, GcInfo};

#[repr(C, align(8))]
pub struct CaptureData {
    gc_info: GcInfo,
    value: Value,
}

#[derive(Clone, Copy)]
pub struct CaptureRef(pub NonNull<CaptureData>);

impl CaptureRef {
    pub fn new(value: Value, gc: &mut GarbageCollector) -> Self {
        let this = CaptureRef(
            NonNull::new(Box::into_raw(Box::new(CaptureData {
                gc_info: GcInfo::default(),
                value,
            })))
            .unwrap(),
        );
        gc.start_tracking(Value::from(this), size_of::<CaptureData>());
        this
    }

    pub fn value(&self) -> Value {
        unsafe { self.0.as_ref().value }
    }

    pub fn value_mut(&mut self) -> &mut Value {
        unsafe { &mut self.0.as_mut().value }
    }

    pub fn gc_mark(&self, gc: &mut GarbageCollector) {
        let data = unsafe { self.0.as_ref() };
        if data.gc_info.mark() {
            return;
        }
        gc.mark(data.value);
    }

    pub fn gc_sweep(&mut self) -> bool {
        if unsafe { self.0.as_ref() }.gc_info.unmark() {
            return true;
        }
        drop(unsafe { Box::<CaptureData>::from_raw(self.0.as_ptr()) });
        false
    }
}
