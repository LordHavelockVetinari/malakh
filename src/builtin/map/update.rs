use crate::builtin::helper::{Action, Method, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::value::hashable::HashableValue;
use crate::vm::{Value, Vm};

pub struct Update {
    current_key: Option<HashableValue>,
}

impl Method for Update {
    type Parent = super::Map;

    fn new(_parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        (Self { current_key: None }, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(key) = self.current_key {
            gc.mark(key.get());
        }
    }

    fn input(&mut self, input: Value, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        if let Some(key) = self.current_key.take() {
            parent.entry(key).insert((key, input));
            Action::Input
        } else {
            let Some(key) = HashableValue::new(input) else {
                err!(vm, "non-hashable key: {:?}", input);
            };
            self.current_key = Some(key);
            Action::Input
        }
    }
}
