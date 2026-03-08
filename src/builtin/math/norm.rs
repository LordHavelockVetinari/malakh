use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Norm {
    sum: f64,
}

impl Function for Norm {
    const NAME: &str = "Norm";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self { sum: 0.0 }, Action::OptionalInput)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            err!(vm, "type error: <norm> {}", input.type_name());
        };
        self.sum += x * x;
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        Action::Output(Value::alloc_from(self.sum.sqrt(), vm.gc_mut()))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
