use crate::builtin::helper::{self, Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessRef;
use crate::vm::{Value, Vm};

pub struct AssertErr {
    symbol: Option<Value>,
}

impl Function for AssertErr {
    const NAME: &str = "AssertErr";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self { symbol: None }, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(symbol) = self.symbol {
            gc.mark(symbol);
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::Action {
        if let Some(proc) = input.as_any_process_ref() {
            let Some(error) = proc.error(vm) else {
                err!(
                    vm,
                    "assertion error: process in {} state; expected .Err state",
                    proc.state(),
                );
            };
            if let Some(symbol) = self.symbol
                && !error.matches(symbol)
            {
                err!(vm, cause = error);
            }
            Action::Stop
        } else {
            if let Some(symbol) = self.symbol {
                err!(
                    vm,
                    "type error: {} {} {}",
                    Self::NAME,
                    symbol.type_name(),
                    input.type_name(),
                );
            }
            self.symbol = Some(input);
            Action::Input
        }
    }
}
