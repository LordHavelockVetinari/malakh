use crate::builtin::helper::{Action, Method};
use crate::vm::Vm;
use crate::vm::gc::GarbageCollector;

pub struct Keys {
    iter: super::Iter,
}

impl Method for Keys {
    type Parent = super::Map;

    fn new(parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let mut this = Self {
            iter: parent.iter(),
        };
        if let Some((key, _)) = this.iter.next(parent) {
            (this, Action::Output(key.get()))
        } else {
            (this, Action::Stop)
        }
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn after_output(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if let Some((key, _)) = self.iter.next(parent) {
            Action::Output(key.get())
        } else {
            Action::Stop
        }
    }
}
