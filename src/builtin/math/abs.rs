use either::Either::{Left, Right};

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Abs;

impl Function for Abs {
    const NAME: &str = "Abs";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if let Some(x) = input.as_f64() {
            if x.is_sign_negative() {
                Action::Output(Value::alloc_from(-x, vm.gc_mut()))
            } else {
                Action::Output(input)
            }
        } else if let Some(either) = input.as_int() {
            let result = match either {
                Left(n) if n >= 0 => input,
                Left(n) => Value::alloc_from(-n, vm.gc_mut()),
                Right(n) if *n >= 0 => input,
                Right(n) => Value::alloc_from(-n, vm.gc_mut()),
            };
            Action::Output(result)
        } else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        }
    }
}
