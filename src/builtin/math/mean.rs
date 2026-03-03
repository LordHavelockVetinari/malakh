use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Mean {
    sum: f64,
    count: f64,
}

impl helper::BasicAggregator for Mean {
    const NAME: &str = "Mean";

    fn new(_vm: &mut Vm) -> Self {
        Self {
            sum: 0.0,
            count: 0.0,
        }
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn get(&mut self, vm: &mut Vm) -> Option<Value> {
        if self.count == 0.0 {
            return None;
        }
        Some(Value::alloc_from(self.sum / self.count, vm.gc_mut()))
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        let Some(x) = value.number_to_f64() else {
            todo!("type error");
        };
        self.sum += x;
        self.count += 1.0;
    }
}
