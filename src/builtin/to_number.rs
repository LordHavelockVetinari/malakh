use std::str::FromStr;

use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct ToNumber;

impl Function for ToNumber {
    const NAME: &str = "ToNumber";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if input.is_number() {
            return Action::Output(input);
        }
        let Some(s) = input.as_string_ref() else {
            panic!("ToNumber got bad argument");
        };
        let Ok(s) = str::from_utf8(s.bytes()) else {
            return Action::Stop;
        };
        if let Some(n) = super::to_int::string_to_int(s) {
            Action::Output(Value::alloc_from(n, vm.gc_mut()))
        } else if let Ok(x) = f64::from_str(s) {
            Action::Output(Value::alloc_from(x, vm.gc_mut()))
        } else {
            Action::Stop
        }
    }
}
