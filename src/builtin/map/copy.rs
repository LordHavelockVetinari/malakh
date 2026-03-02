use crate::builtin::helper::{Action, Method};
use crate::vm::Value;
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::gc::GarbageCollector;

pub struct Copy;

impl Method for Copy {
    type Parent = super::Map;

    fn new(parent: &mut Self::Parent, vm: &mut crate::vm::Vm) -> (Self, Action) {
        let family_index = *super::FAMILY_INDEX
            .get()
            .expect("Map index should have been initialized");
        let family = vm.get_builtin_family(family_index);
        let mut result = BuiltinProcessRef::new(family, None, vm);
        let result_data = unsafe { result.data_mut::<super::Map>() };
        result_data.data = parent.data.clone();
        let result = Value::from(result);
        (Self, Action::Output(result))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
