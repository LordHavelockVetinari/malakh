use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessRef};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::{Value, Vm};

pub enum Action {
    Input,
    OptionalInput,
    Output(Value),
    Stop,
}

pub trait Function: Sized {
    const NAME: &str;

    fn new(vm: &mut Vm) -> (Self, Action);

    fn gc_mark_content(&self, gc: &mut GarbageCollector);

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        _ = (input, vm);
        panic!("function shouldn't get input")
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        _ = vm;
        panic!("function shouldn't request input")
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        let _ = vm;
        Action::Stop
    }
}

#[repr(transparent)]
pub struct AsFunction<T>(T);

impl<T: Function> BuiltinProcessData for AsFunction<T> {
    const NAME: &str = <T as Function>::NAME;

    unsafe fn init(mut process: BuiltinProcessRef, parent: Option<BuiltinProcessRef>, vm: &mut Vm) {
        debug_assert!(parent.is_none());
        let (this, action) = T::new(vm);
        unsafe {
            process.data_ptr::<T>().write(this);
        }
        match action {
            Action::Input => *process.state_mut() = ProcessState::In,
            Action::OptionalInput => *process.state_mut() = ProcessState::OptIn,
            Action::Output(x) => {
                *process.state_mut() = ProcessState::Out;
                *process.output_slot_mut() = x;
            }
            Action::Stop => *process.state_mut() = ProcessState::Stop,
        }
    }

    unsafe fn gc_mark_content(process: BuiltinProcessRef, gc: &mut GarbageCollector) {
        unsafe { process.data::<T>() }.gc_mark_content(gc);
    }

    unsafe fn enter(
        mut process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        use Action::*;
        let state = process.state();
        let this = unsafe { process.data_mut::<T>() };
        match state {
            ProcessState::In => {
                let input = input.expect("process in .In state running without input");
                match this.input(input, vm) {
                    Input => {}
                    OptionalInput => {
                        *process.state_mut() = ProcessState::OptIn;
                    }
                    Output(output) => {
                        *process.output_slot_mut() = output;
                        *process.state_mut() = ProcessState::Out;
                    }
                    Stop => {
                        *process.state_mut() = ProcessState::Stop;
                    }
                }
            }
            ProcessState::OptIn => {
                let action = if let Some(input) = input {
                    this.input(input, vm)
                } else {
                    this.no_input(vm)
                };
                match action {
                    Input => {
                        *process.state_mut() = ProcessState::In;
                    }
                    OptionalInput => {}
                    Output(output) => {
                        *process.output_slot_mut() = output;
                        *process.state_mut() = ProcessState::Out;
                    }
                    Stop => {
                        *process.state_mut() = ProcessState::Stop;
                    }
                }
            }
            ProcessState::Out => match this.after_output(vm) {
                Input => {
                    *process.state_mut() = ProcessState::In;
                }
                OptionalInput => {
                    *process.state_mut() = ProcessState::OptIn;
                }
                Output(output) => {
                    *process.output_slot_mut() = output;
                }
                Stop => {
                    *process.state_mut() = ProcessState::Stop;
                }
            },
            _ => unreachable!("trying to run function process in state {:?}", state),
        }
        process
    }
}

pub trait BasicAggregator {
    const NAME: &str;
    fn new(vm: &mut Vm) -> Self;
    fn gc_mark_content(&self, gc: &mut GarbageCollector);
    fn get(&mut self, vm: &mut Vm) -> Option<Value>;
    fn put(&mut self, value: Value, vm: &mut Vm);
}

#[repr(transparent)]
pub struct AggregatorToFunction<T>(T);

impl<T: BasicAggregator> Function for AggregatorToFunction<T> {
    const NAME: &str = <T as BasicAggregator>::NAME;

    fn new(vm: &mut Vm) -> (Self, Action) {
        (AggregatorToFunction(T::new(vm)), Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        self.0.gc_mark_content(gc);
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        self.0.put(input, vm);
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        if let Some(output) = self.0.get(vm) {
            Action::Output(output)
        } else {
            Action::OptionalInput
        }
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}

pub type AsBasicAggregator<T> = AsFunction<AggregatorToFunction<T>>;

pub trait Method: Sized {
    type Parent: BuiltinProcessData;

    fn new(parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action);

    fn gc_mark_content(&self, gc: &mut GarbageCollector);

    fn input(&mut self, input: Value, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        _ = (input, parent, vm);
        panic!("method shouldn't get input")
    }

    fn no_input(&mut self, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        _ = (parent, vm);
        panic!("method shouldn't request input")
    }

    fn after_output(&mut self, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        let _ = (parent, vm);
        Action::Stop
    }
}

#[repr(C)]
pub struct AsMethod<T> {
    parent: BuiltinProcessRef,
    data: T,
}

impl<T: Method> BuiltinProcessData for AsMethod<T> {
    const NAME: &str = T::Parent::NAME;

    unsafe fn init(mut process: BuiltinProcessRef, parent: Option<BuiltinProcessRef>, vm: &mut Vm) {
        let mut parent = parent.expect("child process should have a parent");
        let parent_data = unsafe { parent.data_mut::<T::Parent>() };
        let (data, action) = T::new(parent_data, vm);
        unsafe {
            process
                .data_ptr::<AsMethod<T>>()
                .write(AsMethod { parent, data });
        }
        match action {
            Action::Input => *process.state_mut() = ProcessState::In,
            Action::OptionalInput => *process.state_mut() = ProcessState::OptIn,
            Action::Output(x) => {
                *process.state_mut() = ProcessState::Out;
                *process.output_slot_mut() = x;
            }
            Action::Stop => *process.state_mut() = ProcessState::Stop,
        }
    }

    unsafe fn gc_mark_content(process: BuiltinProcessRef, gc: &mut GarbageCollector) {
        let &Self { parent, ref data } = unsafe { process.data::<Self>() };
        gc.mark(Value::from(parent));
        <T as Method>::gc_mark_content(data, gc);
    }

    unsafe fn enter(
        mut process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        use Action::*;
        let state = process.state();
        let &mut Self {
            mut parent,
            ref mut data,
        } = unsafe { process.data_mut::<Self>() };
        let parent = unsafe { parent.data_mut::<T::Parent>() };
        match state {
            ProcessState::In => {
                let input = input.expect("process in .In state running without input");
                match data.input(input, parent, vm) {
                    Input => {}
                    OptionalInput => {
                        *process.state_mut() = ProcessState::OptIn;
                    }
                    Output(output) => {
                        *process.output_slot_mut() = output;
                        *process.state_mut() = ProcessState::Out;
                    }
                    Stop => {
                        *process.state_mut() = ProcessState::Stop;
                    }
                }
            }
            ProcessState::OptIn => {
                let action = if let Some(input) = input {
                    data.input(input, parent, vm)
                } else {
                    data.no_input(parent, vm)
                };
                match action {
                    Input => {
                        *process.state_mut() = ProcessState::In;
                    }
                    OptionalInput => {}
                    Output(output) => {
                        *process.output_slot_mut() = output;
                        *process.state_mut() = ProcessState::Out;
                    }
                    Stop => {
                        *process.state_mut() = ProcessState::Stop;
                    }
                }
            }
            ProcessState::Out => match data.after_output(parent, vm) {
                Input => {
                    *process.state_mut() = ProcessState::In;
                }
                OptionalInput => {
                    *process.state_mut() = ProcessState::OptIn;
                }
                Output(output) => {
                    *process.output_slot_mut() = output;
                }
                Stop => {
                    *process.state_mut() = ProcessState::Stop;
                }
            },
            _ => unreachable!("trying to run function process in state {:?}", state),
        }
        process
    }
}

macro_rules! _define_class_symbol {
    ($symbol:ident, $name:ident, $T:ty,) => {
        if $symbol == $crate::vm::symbol::Symbol::$name {
            return Some(*methods::$name.get().expect("method should be initialized"));
        }
    };
    ($symbol:ident, $name:ident, $T:ty, no_symbol) => {};
}

macro_rules! define_class {
    ($($(#[$meta:ident])? $name:ident => $T:ty),* $(,)?) => {
        mod methods {
            $(
                pub static $name: ::std::sync::OnceLock<u32> = ::std::sync::OnceLock::new();
            )*
        }

        pub static FAMILY_INDEX: ::std::sync::OnceLock<u32> = ::std::sync::OnceLock::new();

        pub(in $crate::builtin) fn init(#[allow(unused)] collector: &mut $crate::builtin::BuiltinCollector, family_index: u32) {
            FAMILY_INDEX.set(family_index).expect("family initialized twice");
            $({
                let index = collector.add_family(
                    $crate::vm::builtin_process::BuiltinProcessFamily::from_type::<$crate::builtin::helper::AsMethod<$T>>(),
                    $crate::builtin::NO_PATH
                );
                methods::$name.set(index).expect("method created twice");
            })*
        }

        #[allow(unused)]
        fn symbol_to_method_index(symbol: &'static $crate::vm::symbol::Symbol) -> Option<u32> {
            $(
                $crate::builtin::helper::_define_class_symbol!(symbol, $name, $T, $($meta)?);
            )*
            None
        }
    };
}

pub(crate) use {_define_class_symbol, define_class};
