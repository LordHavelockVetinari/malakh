use std::io::{self, Write};
use std::ptr;
use std::rc::Rc;

use crate::builtin::runtime_error;
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Instruction, Value, Vm};

#[derive(Debug, thiserror::Error)]
#[error("type error")]
pub struct TypeError;

#[derive(Default)]
pub struct ErrorData {
    values: Rc<Vec<Value>>,
    instruction_pointer: *const Instruction,
    cause: Option<ErrorRef>,
}

#[derive(Clone, Copy)]
pub struct ErrorRef(BuiltinProcessRef);

impl ErrorRef {
    pub unsafe fn from_builtin_process(inner: BuiltinProcessRef, vm: &Vm) -> Self {
        debug_assert!(ptr::eq(
            inner.family(),
            vm.get_builtin_family(*runtime_error::FAMILY_INDEX.get().unwrap())
        ));
        Self(inner)
    }

    pub unsafe fn from_value(inner: Value, vm: &Vm) -> Self {
        let inner = inner
            .as_builtin_process_ref()
            .expect("RuntimeError is always a builtin process");
        unsafe { Self::from_builtin_process(inner, vm) }
    }

    fn data(&self) -> &ErrorData {
        unsafe { self.0.data::<ErrorData>() }
    }

    fn data_mut(&mut self) -> &mut ErrorData {
        unsafe { self.0.data_mut::<ErrorData>() }
    }

    pub fn values(&self) -> &[Value] {
        &self.data().values
    }

    fn values_mut(&mut self) -> &mut Rc<Vec<Value>> {
        &mut self.data_mut().values
    }

    pub fn instruction_pointer(&self) -> *const Instruction {
        self.data().instruction_pointer
    }

    fn instruction_pointer_mut(&mut self) -> &mut *const Instruction {
        &mut self.data_mut().instruction_pointer
    }

    pub fn cause(&self) -> Option<Self> {
        self.data().cause
    }

    fn cause_mut(&mut self) -> &mut Option<Self> {
        &mut self.data_mut().cause
    }

    pub fn new(vm: &mut Vm) -> Self {
        let mut this = unsafe { Self::from_builtin_process(runtime_error::new_process(vm), vm) };
        *this.instruction_pointer_mut() = vm.instruction_pointer();
        this
    }

    pub fn new_propagated(vm: &mut Vm, cause: Self) -> Self {
        let mut this = Self::new(vm);
        *this.values_mut() = Rc::clone(&cause.data().values);
        *this.cause_mut() = Some(cause);
        this
    }

    pub fn reserve(&mut self, additional: usize) {
        Rc::make_mut(self.values_mut()).reserve(additional);
    }

    pub fn extend(&mut self, value: Value) {
        Rc::make_mut(self.values_mut()).push(value);
    }

    pub fn from_string(vm: &mut Vm, s: &str) -> Self {
        let mut error = Self::new(vm);
        error.extend(Value::string_from_bytes(s.as_bytes(), vm.gc_mut()));
        error
    }

    pub fn pretty_print<W: Write>(&self, output: &mut W) -> Result<(), io::Error> {
        let mut parts = Vec::new();
        parts.push(*self);
        while let Some(cause) = parts.last().unwrap().cause() {
            parts.push(cause);
        }
        for (i, part) in parts.iter().rev().enumerate() {
            if i == 0 {
                write!(output, "error:")?;
                if let [value] = part.values()
                    && value.is_string()
                {
                    write!(output, " ")?;
                    value.write_to(output)?;
                } else {
                    for value in part.values() {
                        write!(output, " {:?}", value)?;
                    }
                }
                writeln!(output)?;
                write!(output, "    thrown at:   ")?;
            } else {
                write!(output, "    rethrown at: ")?;
            }
            writeln!(output, "{:p}", part.instruction_pointer())?;
        }
        Ok(())
    }

    pub fn matches(&self, value: Value) -> bool {
        self.values().contains(&value)
    }
}

impl From<ErrorRef> for BuiltinProcessRef {
    fn from(error: ErrorRef) -> Self {
        error.0
    }
}

impl From<ErrorRef> for Value {
    fn from(value: ErrorRef) -> Self {
        Value::from(BuiltinProcessRef::from(value))
    }
}

impl ErrorData {
    pub fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        for &value in &**self.values {
            gc.mark(value);
        }
        if let Some(cause) = self.cause {
            gc.mark(Value::from(cause));
        }
    }
}
