use malachite::Integer;
use malachite::base::num::arithmetic::traits::{Mod, Parity};
use malachite::base::num::basic::traits::One;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct ModPow {
    base: Option<Integer>,
    exp: Option<Integer>,
}

impl Function for ModPow {
    const NAME: &str = "ModPow";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            base: None,
            exp: None,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(base) = &self.base else {
            let Some(base) = input.int_to_integer() else {
                err!(vm, "type error: {} {}", Self::NAME, input.type_name(),);
            };
            self.base = Some(base);
            return Action::Input;
        };
        let Some(mut exp) = self.exp.take() else {
            let Some(exp) = input.int_to_integer() else {
                err!(
                    vm,
                    "type error: {} {int} {}",
                    Self::NAME,
                    input.type_name(),
                    int = Value::ZERO.type_name(),
                );
            };
            if exp < 0 {
                err!(vm, "{} got negative exponent", Self::NAME);
            }
            self.exp = Some(exp);
            return Action::Input;
        };
        let Some(modulus) = input.int_to_integer() else {
            err!(
                vm,
                "type error: {} {int} {int} {}",
                Self::NAME,
                input.type_name(),
                int = Value::ZERO.type_name(),
            );
        };
        if modulus <= 0 {
            err!(vm, "{} got nonpositive modulus", Self::NAME);
        }
        if modulus == 1 {
            return Action::Output(Value::ZERO);
        }
        let mut result = Integer::ONE;
        let mut base = base.mod_op(&modulus);
        while exp > 0 {
            if exp.odd() {
                result = result * &base % &modulus;
            }
            base = &base * &base % &modulus;
            exp >>= 1;
        }
        Action::Output(Value::alloc_from(result, vm.gc_mut()))
    }
}
