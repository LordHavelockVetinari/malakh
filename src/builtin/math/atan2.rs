use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Atan2 {
    arg1: Option<f64>,
}

impl Function for Atan2 {
    const NAME: &str = "Atan2";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self { arg1: None };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            todo!("type error");
        };
        if let Some(y) = self.arg1 {
            Action::Output(Value::from_f64(y.atan2(x), vm.gc_mut()))
        } else {
            self.arg1 = Some(x);
            Action::Input
        }
    }
}
