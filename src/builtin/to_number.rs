use std::str::FromStr;

use crate::builtin::helper::{self, BasicFunction};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct ToNumber;

impl BasicFunction for ToNumber {
    const NAME: &str = "ToNumber";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::BasicFunctionResult {
        use helper::BasicFunctionResult::*;
        if input.is_number() {
            return Output(input);
        }
        let Some(s) = input.as_string_ref() else {
            panic!("ToNumber got bad argument");
        };
        let Ok(s) = str::from_utf8(s.bytes()) else {
            return Stop;
        };
        if let Some(n) = super::to_int::string_to_int(s) {
            Output(Value::from_integer(n, vm.gc_mut()))
        } else if let Ok(x) = f64::from_str(s) {
            Output(Value::from_f64(x, vm.gc_mut()))
        } else {
            Stop
        }
    }
}
