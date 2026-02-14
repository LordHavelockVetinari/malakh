use std::cell::Cell;
use std::ops::ControlFlow;

use crate::vm::Value;

#[derive(Default)]
#[repr(C)]
pub struct GcInfo {
    is_marked: Cell<bool>,
}

#[derive(Debug)]
pub struct GarbageCollector {
    // All the ProcessRef's, StringRef's, BigIntRef's, and FloatRef's.
    tracked_values: Vec<Value>,
    // May contain only ProcessRef's, StringRef's, BigIntRef's, and FloatRef's.
    to_mark: Vec<Value>,
    new_mem_size: usize,
    collection_threshold: usize,
}

impl GcInfo {
    // Returns true if the process was previously marked.
    pub fn mark(&self) -> bool {
        self.is_marked.replace(true)
    }

    // Returns true if the process was previously marked.
    pub fn unmark(&self) -> bool {
        self.is_marked.replace(false)
    }
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            tracked_values: Vec::new(),
            to_mark: Vec::new(),
            new_mem_size: 0,
            collection_threshold: 0x10_0000,
        }
    }

    // If value is a string, its content might be uninitialized.
    pub fn start_tracking(&mut self, value: Value, size: usize) {
        self.tracked_values.push(value);
        self.new_mem_size += size;
    }

    fn mark_step(&mut self) -> ControlFlow<()> {
        let Some(value) = self.to_mark.pop() else {
            return ControlFlow::Break(());
        };
        match value.tag() {
            Value::USER_PROCESS_TAG => {
                value.as_user_process_ref().unwrap().gc_mark(self);
            }
            Value::BUILTIN_PROCESS_TAG => {
                value.as_builtin_process_ref().unwrap().gc_mark(self);
            }
            Value::STRING_TAG => {
                value.as_string_ref().unwrap().gc_mark();
            }
            Value::BIG_INT_TAG => {
                value.as_big_int_ref().unwrap().gc_mark();
            }
            Value::FLOAT_TAG => {
                value.as_float_ref().unwrap().gc_mark();
            }
            Value::CAPTURE_TAG => value.as_capture_ref().unwrap().gc_mark(self),
            Value::SMALL_INT_TAG | Value::SYMBOL_TAG | 8.. => unreachable!(),
        }
        ControlFlow::Continue(())
    }

    pub fn mark(&mut self, value: Value) {
        match value.tag() {
            Value::SMALL_INT_TAG | Value::SYMBOL_TAG => {}
            Value::USER_PROCESS_TAG
            | Value::BUILTIN_PROCESS_TAG
            | Value::STRING_TAG
            | Value::BIG_INT_TAG
            | Value::FLOAT_TAG
            | Value::CAPTURE_TAG => {
                self.to_mark.push(value);
            }
            8.. => unreachable!(),
        }
    }

    fn sweep(&mut self) {
        self.tracked_values.retain(|value| match value.tag() {
            Value::USER_PROCESS_TAG => value.as_user_process_ref().unwrap().gc_sweep(),
            Value::BUILTIN_PROCESS_TAG => value.as_builtin_process_ref().unwrap().gc_sweep(),
            Value::STRING_TAG => value.as_string_ref().unwrap().gc_sweep(),
            Value::BIG_INT_TAG => value.as_big_int_ref().unwrap().gc_sweep(),
            Value::FLOAT_TAG => value.as_float_ref().unwrap().gc_sweep(),
            Value::CAPTURE_TAG => value.as_capture_ref().unwrap().gc_sweep(),
            Value::SMALL_INT_TAG | Value::SYMBOL_TAG | 8.. => unreachable!(),
        });
    }

    pub fn collect(&mut self) {
        while self.mark_step().is_continue() {}
        self.sweep();
        self.new_mem_size = 0;
    }

    pub fn should_collect(&self) -> bool {
        self.new_mem_size >= self.collection_threshold
    }
}
