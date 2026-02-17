use std::f64::consts;

use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Log {
    arg1: Option<f64>,
}

impl Function for Log {
    const NAME: &str = "Log";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self { arg1: None };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            todo!("type error");
        };
        if let Some(base) = self.arg1 {
            let result = match base {
                2.0 => x.log2(),
                10.0 => x.log10(),
                consts::E => x.ln(),
                _ => x.log(base),
            };
            Action::Output(Value::from_f64(result, vm.gc_mut()))
        } else {
            self.arg1 = Some(x);
            Action::OptionalInput
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let x = self.arg1.expect("arg1 should be initialized");
        Action::Output(Value::from_f64(x.ln(), vm.gc_mut()))
    }
}
