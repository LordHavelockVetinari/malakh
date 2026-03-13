mod error;
mod int;
mod seed;
mod set_seed;
mod uniform;
mod urn;

use crate::builtin::helper::{self, new_error};
use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessRef};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::{Value, Vm};

pub struct Random {
    rng: fastrand::Rng,
}

helper::define_class!(
    #[no_symbol]
    UNIFORM => self::uniform::Uniform,
    #[no_symbol]
    ERROR => self::error::Error,
    INT => self::int::Int,
    URN => self::urn::Urn,
    SEED => self::seed::Seed,
    SET_SEED => self::set_seed::SetSeed,
);

impl BuiltinProcessData for Random {
    const NAME: &str = "Random";

    unsafe fn init(
        mut process: BuiltinProcessRef,
        parent: Option<BuiltinProcessRef>,
        _vm: &mut Vm,
    ) {
        debug_assert!(parent.is_none());
        unsafe {
            process.data_ptr::<Self>().write(Self {
                rng: fastrand::Rng::new(),
            });
        }
        *process.state_mut() = ProcessState::ForkIn;
    }

    unsafe fn enter(
        process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        let input = input.expect("Random process didn't get input");
        if input.is_number() {
            vm.put_temporary1(input);
            let index = *methods::UNIFORM.get().expect("Random method uninitialized");
            let family = vm.get_builtin_family(index);
            return BuiltinProcessRef::new(family, Some(process), vm);
        }
        let Some(cmd) = input.as_symbol() else {
            let error = new_error!(vm, "type error: <random> {}", input.type_name());
            vm.put_temporary1(Value::from(error));
            let index = *methods::ERROR.get().expect("Random method uninitialized");
            let family = vm.get_builtin_family(index);
            return BuiltinProcessRef::new(family, Some(process), vm);
        };
        let Some(index) = symbol_to_method_index(cmd) else {
            let error = new_error!(vm, "undefined method: <random> {:?}", input);
            vm.put_temporary1(Value::from(error));
            let index = *methods::ERROR.get().expect("Random method uninitialized");
            let family = vm.get_builtin_family(index);
            return BuiltinProcessRef::new(family, Some(process), vm);
        };
        let family = vm.get_builtin_family(index);
        BuiltinProcessRef::new(family, Some(process), vm)
    }

    unsafe fn gc_mark_content(_process: BuiltinProcessRef, _gc: &mut GarbageCollector) {}
}
