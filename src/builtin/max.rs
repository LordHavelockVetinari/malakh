use std::cmp::Ordering;

use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Max {
    accumulator: Option<Value>,
}

impl helper::BasicAggregator for Max {
    const NAME: &str = "Max";

    fn new(_vm: &mut Vm) -> Self {
        Self { accumulator: None }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(acc) = self.accumulator {
            gc.mark(acc);
        }
    }

    fn get(&mut self, _vm: &mut Vm) -> Option<Value> {
        self.accumulator
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        if let Some(acc) = &mut self.accumulator {
            let Ok(ord) = acc.compare(value) else {
                todo!("type error");
            };
            if ord == Some(Ordering::Less) || (ord.is_none() && !acc.is_nan()) {
                #[cfg(debug_assertions)]
                {
                    if ord.is_none() && !acc.is_nan() {
                        assert!(value.is_nan());
                    }
                }
                *acc = value;
            }
        } else {
            if !(value.is_number() || value.is_string()) {
                todo!("type error");
            }
            self.accumulator = Some(value);
        }
    }
}
