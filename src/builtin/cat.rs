use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct Cat {
    buffer: StringBuffer,
}

impl helper::BasicAggregator for Cat {
    const NAME: &str = "Cat";

    fn new(vm: &mut Vm) -> Self {
        Self {
            buffer: StringBuffer::new(vm.gc_mut()),
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
        value.write_to(&mut writer).unwrap();
    }
}
