mod clear;
mod copy;
mod each;
mod index;
mod length;
pub mod of;
mod pop;
mod push;

use crate::builtin::helper;
use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessRef};
use crate::vm::process::ProcessState;
use crate::vm::{Value, Vm};

pub struct List {
    data: Vec<Value>,
}

helper::define_class!(
    LENGTH => self::length::Length,
    PUSH => self::push::Push,
    POP => self::pop::Pop,
    EACH => self::each::Each,
    CLEAR => self::clear::Clear,
    COPY => self::copy::Copy,
    #[no_symbol]
    INDEX => self::index::Index,
);

impl BuiltinProcessData for List {
    const NAME: &str = "List";

    unsafe fn init(
        mut process: BuiltinProcessRef,
        parent: Option<BuiltinProcessRef>,
        _vm: &mut Vm,
    ) {
        debug_assert!(parent.is_none());
        unsafe {
            process.data_ptr::<Self>().write(Self { data: Vec::new() });
        }
        *process.state_mut() = ProcessState::ForkIn;
    }

    unsafe fn enter(
        process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        let input = input.expect("List process didn't get input");
        if input.is_int() {
            vm.put_temporary1(input);
            let index = *methods::INDEX.get().expect("List method uninitialized");
            let family = vm.get_builtin_family(index);
            return BuiltinProcessRef::new(family, Some(process), vm);
        }
        let Some(cmd) = input.as_symbol() else {
            todo!("List expected a symbol or an integer");
        };
        let Some(index) = symbol_to_method_index(cmd) else {
            todo!("invalid list method");
        };
        let family = vm.get_builtin_family(index);
        BuiltinProcessRef::new(family, Some(process), vm)
    }

    unsafe fn gc_mark_content(
        process: BuiltinProcessRef,
        gc: &mut crate::vm::gc::GarbageCollector,
    ) {
        let this = unsafe { process.data::<Self>() };
        for &item in &this.data {
            gc.mark(item);
        }
    }
}
