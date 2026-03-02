use crate::builtin::helper::{self, Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct IsFinite;

impl Function for IsFinite {
    const NAME: &str = "IsFinite";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::Action {
        let result = input.as_f64().is_some_and(f64::is_finite) || input.is_int();
        Action::Output(Value::from(result))
    }
}
