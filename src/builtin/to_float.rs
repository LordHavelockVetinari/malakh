use std::str::FromStr;

use crate::builtin::helper::{self, BasicFunction};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct ToFloat;

impl BasicFunction for ToFloat {
    const NAME: &str = "ToFloat";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::BasicFunctionResult {
        use helper::BasicFunctionResult::*;
        if input.is_float() {
            return Output(input);
        }
        let result = if let Some(result) = input.number_to_f64() {
            result
        } else {
            let Some(s) = input.as_string_ref() else {
                todo!("ToFloat got bad argument");
            };
            let s = s.bytes();
            if !s.is_ascii() {
                return Stop;
            }
            let s = str::from_utf8(s).unwrap();
            let Ok(result) = f64::from_str(s) else {
                return Stop;
            };
            result
        };
        Output(Value::alloc_from(result, vm.gc_mut()))
    }
}
