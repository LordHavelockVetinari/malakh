use std::io::Write;

use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct FromBytes {
    buffer: StringBuffer,
}

impl helper::BasicAggregator for FromBytes {
    const NAME: &str = "FromBytes";

    fn new(vm: &mut Vm) -> Self {
        Self {
            buffer: StringBuffer::new(vm.gc_mut()),
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        self.buffer.gc_mark_content(gc);
    }

    fn get(&mut self, vm: &mut Vm) -> Option<Value> {
        Some(Value::from_string_ref(self.buffer.to_string(vm.gc_mut())))
    }

    fn put(&mut self, value: Value, vm: &mut Vm) {
        let Some(n) = value.as_small_int() else {
            todo!("FromBytes got a big integer or another type");
        };
        let Ok(c) = u8::try_from(n) else {
            todo!("FromBytes got a large or negative number");
        };
        self.buffer.writer(vm.gc_mut()).write_all(&[c]).unwrap();
    }
}
