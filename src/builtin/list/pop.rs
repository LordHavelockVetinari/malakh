use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;

pub struct Pop;

impl Method for Pop {
    type Parent = super::List;

    fn new(parent: &mut Self::Parent, _vm: &mut crate::vm::Vm) -> (Self, Action) {
        let Some(result) = parent.data.pop() else {
            todo!(".Pop in empty list");
        };
        (Self, Action::Output(result))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
