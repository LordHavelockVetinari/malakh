use crate::builtin::helper::{Action, Method};
use crate::vm::Value;
use crate::vm::gc::GarbageCollector;
use crate::vm::value::hashable::HashableValue;

pub struct HasKey;

impl Method for HasKey {
    type Parent = super::Map;

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
        let Some(key) = HashableValue::new(input) else {
            return Action::Output(Value::FALSE);
        };
        let result = parent.find_entry(key).is_ok();
        Action::Output(Value::from_bool(result))
    }
}
