use crate::builtin::helper::{Action, Method};
use crate::vm::Value;
use crate::vm::gc::GarbageCollector;

pub struct Length;

impl Method for Length {
    type Parent = super::Map;

    fn new(parent: &mut Self::Parent, vm: &mut crate::vm::Vm) -> (Self, Action) {
        let result = Value::from_usize(parent.data.len(), vm.gc_mut());
        (Self, Action::Output(result))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
