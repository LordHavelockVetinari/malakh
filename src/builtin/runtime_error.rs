use crate::builtin::helper::define_class;
use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessRef};
use crate::vm::error::ErrorData;
use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessState;
use crate::vm::{Value, Vm};

define_class! {}

#[repr(transparent)]
pub struct RuntimeError(ErrorData);

impl BuiltinProcessData for RuntimeError {
    const NAME: &str = "RuntimeError";

    unsafe fn init(
        mut process: BuiltinProcessRef,
        parent: Option<BuiltinProcessRef>,
        _vm: &mut Vm,
    ) {
        assert!(parent.is_none());
        let data = process.data_ptr::<Self>();
        unsafe {
            data.write(Self(ErrorData::default()));
        }
        *process.state_mut() = ProcessState::Stop;
    }

    unsafe fn enter(
        _process: BuiltinProcessRef,
        _vm: &mut Vm,
        _input: Option<Value>,
    ) -> BuiltinProcessRef {
        panic!("RuntimeError process should never be entered");
    }

    unsafe fn gc_mark_content(mut process: BuiltinProcessRef, gc: &mut GarbageCollector) {
        let data = unsafe { process.data_mut::<Self>() };
        data.0.gc_mark_content(gc);
    }
}

pub fn new_process(vm: &mut Vm) -> BuiltinProcessRef {
    let family_index = *FAMILY_INDEX
        .get()
        .expect("RuntimeError family should have been initialized");
    let family = vm.get_builtin_family(family_index);
    BuiltinProcessRef::new(family, None, vm)
}
