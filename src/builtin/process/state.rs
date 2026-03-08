use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessRef;
use crate::vm::{Value, Vm};

pub struct State;

impl Function for State {
    const NAME: &str = "State";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(p) = input.as_any_process_ref() else {
            err!(vm, "type error: {} {:?}", Self::NAME, input);
        };
        Action::Output(Value::from(p.state()))
    }
}
