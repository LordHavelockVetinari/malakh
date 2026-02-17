use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Sin;

impl Function for Sin {
    const NAME: &str = "Sin";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(x) = input.number_to_f64() else {
            todo!("type error");
        };
        Action::Output(Value::from_f64(x.sin(), vm.gc_mut()))
    }
}
