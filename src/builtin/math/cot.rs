use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Cot;

impl Function for Cot {
    const NAME: &str = "Cot";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        Action::Output(Value::alloc_from(x.tan().recip(), vm.gc_mut()))
    }
}
