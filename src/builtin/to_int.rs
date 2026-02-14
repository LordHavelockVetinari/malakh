use std::str::FromStr;

use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

use crate::builtin::helper::{self, BasicFunction};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub fn string_to_int(mut s: &str) -> Option<Integer> {
    if s.starts_with("+") {
        if s.starts_with("+-") {
            return None;
        }
        s = &s[1..];
    }
    Integer::from_str(s).ok()
}

pub struct ToInt;

impl BasicFunction for ToInt {
    const NAME: &str = "ToInt";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::BasicFunctionResult {
        use helper::BasicFunctionResult::*;
        if input.is_int() {
            return Output(input);
        }
        if let Some(x) = input.as_f64() {
            if !x.is_finite() {
                return Stop;
            }
            let (n, _) = Integer::rounding_from(x, RoundingMode::Down);
            return Output(Value::from_integer(n, vm.gc_mut()));
        }
        let Some(s) = input.as_string_ref() else {
            todo!("ToInt got bad argument");
        };
        let bytes = s.bytes();
        let Some(n) = str::from_utf8(bytes).ok().and_then(string_to_int) else {
            return Stop;
        };
        Output(Value::from_integer(n, vm.gc_mut()))
    }
}
