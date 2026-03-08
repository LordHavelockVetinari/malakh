use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Sum {
    accumulator: Value,
}

impl Function for Sum {
    const NAME: &str = "Sum";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            accumulator: Value::ZERO,
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.accumulator);
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Ok(new_sum) = self.accumulator.add(input, vm.gc_mut()) else {
            err!(vm, "type error: <sum> {}", input.type_name());
        };
        self.accumulator = new_sum;
        Action::OptionalInput
    }

    fn no_input(&mut self, _vm: &mut Vm) -> Action {
        Action::Output(self.accumulator)
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
