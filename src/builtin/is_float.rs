use crate::builtin::helper::{self, Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct IsFloat;

impl Function for IsFloat {
    const NAME: &str = "IsFloat";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::Action {
        Action::Output(Value::from_bool(input.is_float()))
    }
}
