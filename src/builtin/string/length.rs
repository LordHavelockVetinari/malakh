use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Length;

impl Function for Length {
    const NAME: &str = "Length";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(s) = input.as_string_ref() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let len = Value::alloc_from(s.bytes().len(), vm.gc_mut());
        Action::Output(len)
    }
}
