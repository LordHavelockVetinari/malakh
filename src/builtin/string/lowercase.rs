use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Lowercase;

impl Function for Lowercase {
    const NAME: &str = "Lowercase";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(s) = input.as_string_ref() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Ok(utf8) = str::from_utf8(s.bytes()) else {
            err!(vm, tag = ENCODING_ERROR, "invalid UTF-8");
        };
        Action::Output(Value::alloc_from(utf8.to_lowercase(), vm.gc_mut()))
    }
}
