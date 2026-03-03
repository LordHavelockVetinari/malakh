use crate::builtin::helper::{Action, Method};
use crate::vm::Value;
use crate::vm::gc::GarbageCollector;

pub struct Length;

impl Method for Length {
    type Parent = super::List;

    fn new(parent: &mut Self::Parent, vm: &mut crate::vm::Vm) -> (Self, Action) {
        let result = Value::alloc_from(parent.data.len(), vm.gc_mut());
        (Self, Action::Output(result))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
