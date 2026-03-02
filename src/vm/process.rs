use either::Either::{self, Left, Right};

use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::error::ErrorRef;
use crate::vm::symbol::Symbol;
use crate::vm::user_process::UserProcessRef;
use crate::vm::{Value, Vm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessState {
    Run,
    Stop,
    In,
    OptIn,
    #[allow(unused)]
    ForkIn,
    Out,
    Err,
}

impl ProcessState {
    pub fn as_value(self) -> Value {
        use ProcessState::*;
        match self {
            Run => Value::from_symbol(Symbol::RUN),
            Stop => Value::from_symbol(Symbol::STOP),
            In => Value::from_symbol(Symbol::IN),
            OptIn => Value::from_symbol(Symbol::OPT_IN),
            ForkIn => Value::from_symbol(Symbol::FORK_IN),
            Out => Value::from_symbol(Symbol::OUT),
            Err => Value::from_symbol(Symbol::ERR),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnyProcessRef(Value);

impl AnyProcessRef {
    pub fn from_value(value: Value) -> Option<Self> {
        if value.is_user_process() || value.is_builtin_process() {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn builtin_or_user_defined(self) -> Either<BuiltinProcessRef, UserProcessRef> {
        if let Some(proc) = self.0.as_user_process_ref() {
            return Right(proc);
        }
        unsafe { Left(self.0.as_builtin_process_ref().unwrap_unchecked()) }
    }

    pub fn state(&self) -> ProcessState {
        match self.builtin_or_user_defined() {
            Left(proc) => proc.state(),
            Right(proc) => proc.state(),
        }
    }

    pub fn output_slot(&self) -> Value {
        match self.builtin_or_user_defined() {
            Left(proc) => proc.output_slot(),
            Right(proc) => proc.output_slot(),
        }
    }

    pub fn error(&self, vm: &mut Vm) -> Option<ErrorRef> {
        if self.state() == ProcessState::Err {
            Some(unsafe { ErrorRef::from_value(self.output_slot(), vm) })
        } else {
            None
        }
    }
}

impl From<BuiltinProcessRef> for AnyProcessRef {
    fn from(builtin_process: BuiltinProcessRef) -> Self {
        Self(Value::from_builtin_process_ref(builtin_process))
    }
}

impl From<UserProcessRef> for AnyProcessRef {
    fn from(user_process: UserProcessRef) -> Self {
        Self(Value::from_user_process_ref(user_process))
    }
}
