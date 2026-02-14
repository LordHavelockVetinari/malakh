use crate::builtin::helper::{Action, Method};
use crate::vm::Value;
use crate::vm::gc::GarbageCollector;

pub struct Push;

impl Method for Push {
    type Parent = super::List;

    fn new(_parent: &mut Self::Parent, _vm: &mut crate::vm::Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(
        &mut self,
        input: Value,
        parent: &mut Self::Parent,
        _vm: &mut crate::vm::Vm,
    ) -> Action {
        parent.data.push(input);
        Action::Input
    }
}
