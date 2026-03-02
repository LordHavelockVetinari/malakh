use std::io::Write;
use std::mem;

use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct FromWords {
    buffer: StringBuffer,
    is_first_word: bool,
}

impl helper::BasicAggregator for FromWords {
    const NAME: &str = "FromWords";

    fn new(vm: &mut Vm) -> Self {
        Self {
            buffer: StringBuffer::new(vm.gc_mut()),
            is_first_word: true,
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        self.buffer.gc_mark_content(gc);
    }

    fn get(&mut self, vm: &mut Vm) -> Option<Value> {
        Some(Value::from(self.buffer.to_string(vm.gc_mut())))
    }

    fn put(&mut self, value: Value, vm: &mut Vm) {
        let mut writer = self.buffer.writer(vm.gc_mut());
        if !mem::take(&mut self.is_first_word) {
            writer.write_all(b" ").unwrap();
        }
        value.write_to(&mut writer).unwrap();
    }
}
