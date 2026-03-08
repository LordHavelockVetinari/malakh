use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Assert;

impl Function for Assert {
    const NAME: &str = "Assert";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(asserted) = input.as_bool() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        if !asserted {
            err!(vm, "assertion error");
        }
        Action::Stop
    }
}
