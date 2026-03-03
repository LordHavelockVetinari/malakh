mod big_int;
pub mod builder;
pub mod builtin_process;
mod capture;
pub mod error;
pub mod examples;
mod float;
pub mod gc;
mod global_variable;
mod instruction;
pub mod macros;
pub mod opcode;
pub mod options;
pub mod process;
pub mod string;
pub mod symbol;
pub mod user_process;
pub mod value;

pub use instruction::Instruction;
use user_process::UserProcessRef;
pub use value::Value;

use crate::vm::builder::VmBuilder;
use crate::vm::builtin_process::{BuiltinProcessFamily, BuiltinProcessRef};
use crate::vm::error::ErrorRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::global_variable::GlobalVariable;
use crate::vm::options::VmOptions;
use crate::vm::process::{AnyProcessRef, ProcessState};
use crate::vm::user_process::UserProcessFamily;

#[derive(Debug)]
pub struct Vm {
    constants: Vec<Value>,
    user_process_families: Vec<&'static UserProcessFamily>,
    builtin_process_families: &'static [&'static BuiltinProcessFamily],
    global_variables: Vec<&'static GlobalVariable>,
    call_stack: Vec<UserProcessRef>,
    instruction_pointer: *const Instruction,
    memory: *mut [Value],
    gc: GarbageCollector,
    // A place where any builtin process can put temporary data.
    temporary1: Option<Value>,
    options: VmOptions,
}

impl Vm {
    pub fn builder() -> VmBuilder {
        VmBuilder::new()
    }

    pub fn get_builtin_family(&self, index: u32) -> &'static BuiltinProcessFamily {
        self.builtin_process_families[index as usize]
    }

    pub fn instruction_pointer(&self) -> *const Instruction {
        self.instruction_pointer
    }

    pub fn register(&self, n: u16) -> Value {
        unsafe { self.memory.cast::<Value>().add(n as usize).read() }
    }

    pub fn register_mut(&mut self, n: u16) -> &mut Value {
        unsafe { &mut *self.memory.cast::<Value>().add(n as usize) }
    }

    pub fn put_temporary1(&mut self, value: Value) {
        assert!(self.temporary1.is_none(), "temporary1 should be None");
        self.temporary1 = Some(value);
    }

    pub fn try_take_temporary1(&mut self) -> Option<Value> {
        self.temporary1.take()
    }

    pub fn take_temporary1(&mut self) -> Value {
        self.try_take_temporary1()
            .expect("temporary1 should not be None")
    }

    pub fn assert_temporary1_none(&self) {
        assert!(self.temporary1.is_none());
    }

    pub fn options(&self) -> &VmOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut VmOptions {
        &mut self.options
    }

    pub fn gc_mut(&mut self) -> &mut GarbageCollector {
        &mut self.gc
    }

    fn mark_gc_roots(&mut self) {
        for &c in &self.constants {
            self.gc.mark(c);
        }
        for v in &self.global_variables {
            if let Some(&value) = v.value().get() {
                self.gc.mark(value);
            }
        }
        for &p in &self.call_stack {
            self.gc.mark(Value::from(p));
        }
    }

    // This function is currently called in the following places:
    // - When entering a new process.
    // - After performing a jump.
    fn maybe_collect_garbage(&mut self) {
        if self.gc.should_collect() {
            self.mark_gc_roots();
            self.gc.collect();
        }
    }

    pub fn current_process(&self) -> UserProcessRef {
        *self.call_stack.last().expect("call stack is empty")
    }

    pub fn throw_from_current_process(&mut self, error: ErrorRef) {
        let mut proc = self.current_process();
        *proc.output_slot_mut() = Value::from(error);
        debug_assert!(!proc.take_can_resume());
        let catch_address = proc
            .family()
            .try_bodies
            .iter()
            .find(|try_body| try_body.contains(self.instruction_pointer()))
            .map(|try_body| try_body.end);
        if let Some(catch_address) = catch_address {
            self.jump_absolute(catch_address);
        } else {
            *proc.state_mut() = ProcessState::Err;
            self.pause_user_process();
        }
    }

    fn _propagate_error_inner(&mut self, original_process: AnyProcessRef) {
        let cause = original_process
            .error(self)
            .expect("cannot propagate error unless original process is in .Err state");
        let error = ErrorRef::new_propagated(self, cause);
        self.throw_from_current_process(error);
    }

    pub fn propagate_error<P>(&mut self, original_process: P)
    where
        AnyProcessRef: From<P>,
    {
        self._propagate_error_inner(AnyProcessRef::from(original_process));
    }

    pub fn pause_user_process(&mut self) {
        let mut old_proc = self.call_stack.pop().unwrap();
        *old_proc.instruction_pointer_mut() = self.instruction_pointer;
        let mut new_proc = self.current_process();
        self.instruction_pointer = new_proc.instruction_pointer();
        self.memory = new_proc.memory_mut();
    }

    pub fn enter_user_process(&mut self, mut proc: UserProcessRef) {
        if let Some(old_proc) = self.call_stack.last_mut() {
            *old_proc.instruction_pointer_mut() = self.instruction_pointer;
        }
        self.instruction_pointer = proc.instruction_pointer();
        self.memory = proc.memory_mut();
        self.call_stack.push(proc);
        self.maybe_collect_garbage();
    }

    pub fn enter_builtin_process(
        &mut self,
        proc: BuiltinProcessRef,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        unsafe { (proc.family().enter)(proc, self, input) }
    }

    pub fn jump_absolute(&mut self, target: *const Instruction) {
        self.instruction_pointer = target;
        self.maybe_collect_garbage();
    }

    pub fn jump(&mut self, offset: isize) {
        self.jump_absolute(unsafe { self.instruction_pointer.offset(offset) });
    }

    pub fn step(&mut self) {
        let instruction = unsafe { self.instruction_pointer.read() };
        #[cfg(false)]
        eprintln!(
            "adderess {:p} in {}: {:?}",
            self.instruction_pointer,
            self.call_stack
                .last()
                .map(|&p| format!("{:?}", Value::from_user_process_ref(p)))
                .unwrap_or_else(|| "<not a process>".to_string()),
            instruction
        );
        unsafe {
            self.instruction_pointer = self.instruction_pointer.add(1);
        }
        instruction.run(self);
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
}

macro_rules! throw_from_current_process {
    ($vm:expr, $format:literal $(, $($t:tt)*)?) => {
        {
            let error = $crate::vm::error::ErrorRef::from_string(
                $vm,
                &format!($format $(, $($t)*)?)
            );
            $vm.throw_from_current_process(error);
        }
    }
}

pub(crate) use throw_from_current_process;
