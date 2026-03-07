use crate::builtin::helper::{self, Action, Function};
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
                todo!("assertion error - not in .Err state");
            };
            if let Some(symbol) = self.symbol
                && !error.matches(symbol)
            {
                todo!("assertion error - tag doesn't match")
            }
            Action::Stop
        } else {
            if self.symbol.is_some() {
                todo!("expected a process");
            }
            self.symbol = Some(input);
            Action::Input
        }
    }
}
