use std::cmp::Ordering;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Max {
    accumulator: Option<Value>,
}

impl Function for Max {
    const NAME: &str = "Max";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self { accumulator: None }, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(acc) = self.accumulator {
            gc.mark(acc);
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if let Some(acc) = &mut self.accumulator {
            let Ok(ord) = acc.compare(input) else {
                err!(
                    vm,
                    "type error: Max cannot compare {} and {}",
                    acc.type_name(),
                    input.type_name()
                );
            };
            debug_assert!(ord.is_some() || acc.is_nan() || input.is_nan());
            if ord == Some(Ordering::Less) || (ord.is_none() && !acc.is_nan()) {
                *acc = input;
            }
        } else {
            if !input.is_number() && !input.is_string() {
                err!(vm, "type error: Max {}", input.type_name());
            }
            self.accumulator = Some(input);
        }
        Action::OptionalInput
    }

    fn no_input(&mut self, _vm: &mut Vm) -> Action {
        let output = self
            .accumulator
            .expect("accumulator should be Some during OptIn state");
        Action::Output(output)
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
