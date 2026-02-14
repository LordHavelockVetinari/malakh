use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Stack {
    data: Vec<Value>,
}

impl helper::BasicAggregator for Stack {
    const NAME: &str = "Stack";

    fn new(_vm: &mut Vm) -> Self {
        Self { data: Vec::new() }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        for &value in &self.data {
            gc.mark(value);
        }
    }

    fn get(&mut self, _vm: &mut Vm) -> Option<Value> {
        self.data.pop()
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        self.data.push(value);
    }
}
