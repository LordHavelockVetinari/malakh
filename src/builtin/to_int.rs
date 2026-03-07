use std::str::FromStr;

use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

use crate::builtin::helper::{Action, Function};
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

impl Function for ToInt {
    const NAME: &str = "ToInt";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if input.is_int() {
            return Action::Output(input);
        }
        if let Some(x) = input.as_f64() {
            if !x.is_finite() {
                return Action::Stop;
            }
            let (n, _) = Integer::rounding_from(x, RoundingMode::Down);
            return Action::Output(Value::alloc_from(n, vm.gc_mut()));
        }
        let Some(s) = input.as_string_ref() else {
            todo!("ToInt got bad argument");
        };
        let bytes = s.bytes();
        let Some(n) = str::from_utf8(bytes).ok().and_then(string_to_int) else {
            return Action::Stop;
        };
        Action::Output(Value::alloc_from(n, vm.gc_mut()))
    }
}
