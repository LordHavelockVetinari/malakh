use crate::builtin::helper::{Action, Function};
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

    fn input(&mut self, input: Value, _vm: &mut Vm) -> Action {
        let Some(p) = input.as_any_process_ref() else {
            todo!("Process::State did not get a process");
        };
        Action::Output(Value::from(p.state()))
    }
}
