use crate::builtin::helper::{self, Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Assert;

impl Function for Assert {
    const NAME: &str = "Assert";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::Action {
        let Some(asserted) = input.as_bool() else {
            todo!("asserted value is not a boolean");
        };
        if !asserted {
            todo!("assertion error");
        }
        Action::Stop
    }
}
