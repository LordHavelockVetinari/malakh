use std::collections::VecDeque;

use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Queue {
    data: VecDeque<Value>,
}

impl helper::BasicAggregator for Queue {
    const NAME: &str = "Queue";

    fn new(_vm: &mut Vm) -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        for &value in &self.data {
            gc.mark(value);
        }
    }

    fn get(&mut self, _vm: &mut Vm) -> Option<Value> {
        self.data.pop_front()
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        self.data.push_back(value);
    }
}
