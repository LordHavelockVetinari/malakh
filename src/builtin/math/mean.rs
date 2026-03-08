use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Mean {
    sum: f64,
    count: f64,
}

impl Function for Mean {
    const NAME: &str = "Mean";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            sum: 0.0,
            count: 0.0,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            err!(vm, "type error: <mean> {}", input.type_name());
        };
        self.sum += x;
        self.count += 1.0;
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        Action::Output(Value::alloc_from(self.sum / self.count, vm.gc_mut()))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
