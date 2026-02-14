use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;

pub struct Clear;

impl Method for Clear {
    type Parent = super::List;

    fn new(parent: &mut Self::Parent, _vm: &mut crate::vm::Vm) -> (Self, Action) {
        parent.data.clear();
        (Self, Action::Stop)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
