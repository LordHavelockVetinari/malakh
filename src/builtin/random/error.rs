use crate::builtin::helper::{Action, Method};
use crate::vm::Vm;
use crate::vm::error::ErrorRef;
use crate::vm::gc::GarbageCollector;

pub struct Error;

impl Method for Error {
    type Parent = super::Random;

    fn new(_parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action) {
        let error = vm.take_temporary1();
        let error = unsafe { ErrorRef::from_value(error, vm) };
        (Self, Action::Error(error))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
