use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Norm {
    sum: f64,
}

impl helper::BasicAggregator for Norm {
    const NAME: &str = "Norm";

    fn new(_vm: &mut Vm) -> Self {
        Self { sum: 0.0 }
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn get(&mut self, vm: &mut Vm) -> Option<Value> {
        Some(Value::alloc_from(self.sum.sqrt(), vm.gc_mut()))
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        let Some(x) = value.number_to_f64() else {
            todo!("type error");
        };
        self.sum += x * x;
    }
}
