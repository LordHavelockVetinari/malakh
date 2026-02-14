use either::Either::{self, Left, Right};

use crate::vm::Value;
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::symbol::Symbol;
use crate::vm::user_process::UserProcessRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessState {
    Run,
    Stop,
    Out,
    In,
    OptIn,
    #[allow(unused)]
    ForkIn,
}

impl ProcessState {
    pub fn as_value(self) -> Value {
        use ProcessState::*;
        match self {
            Run => Value::from_symbol(Symbol::RUN),
            Stop => Value::from_symbol(Symbol::STOP),
            Out => Value::from_symbol(Symbol::OUT),
            In => Value::from_symbol(Symbol::IN),
            OptIn => Value::from_symbol(Symbol::OPT_IN),
            ForkIn => Value::from_symbol(Symbol::FORK_IN),
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
}
