use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Round;

impl Function for Round {
    const NAME: &str = "Round";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if let Some(x) = input.as_f64() {
            if !x.is_finite() {
                err!(vm, "{} got non-finite input: {:?}", Self::NAME, input);
            }
            let (n, _) = Integer::rounding_from(x, RoundingMode::Nearest);
            Action::Output(Value::alloc_from(n, vm.gc_mut()))
        } else if input.is_int() {
            Action::Output(input)
        } else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        }
    }
}
