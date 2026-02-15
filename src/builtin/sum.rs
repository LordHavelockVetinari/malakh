use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Sum {
    accumulator: Value,
}

impl helper::BasicAggregator for Sum {
    const NAME: &str = "Sum";

    fn new(_vm: &mut Vm) -> Self {
        Self {
            accumulator: Value::ZERO,
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.accumulator);
    }

    fn get(&mut self, _vm: &mut Vm) -> Option<Value> {
        Some(self.accumulator)
    }

    fn put(&mut self, value: Value, vm: &mut Vm) {
        let Ok(new_sum) = self.accumulator.add(value, vm.gc_mut()) else {
            todo!("type error");
        };
        self.accumulator = new_sum;
    }
}
