use std::mem;

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

pub trait ProcessRef: Sized {
    fn state(&self) -> ProcessState;

    fn state_mut(&mut self) -> &mut ProcessState;

    fn output_slot(&self) -> Value;

    fn output_slot_mut(&mut self) -> &mut Value;

    fn error(&self, vm: &mut Vm) -> Option<ErrorRef> {
        if self.state() == ProcessState::Err {
            Some(unsafe { ErrorRef::from_value(self.output_slot(), vm) })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnyProcessRef(pub Value);

impl From<ProcessState> for &'static Symbol {
    fn from(state: ProcessState) -> Self {
        use ProcessState::*;
        match state {
            Run => Symbol::RUN,
            Stop => Symbol::STOP,
            In => Symbol::IN,
            OptIn => Symbol::OPT_IN,
            ForkIn => Symbol::FORK_IN,
            Out => Symbol::OUT,
            Err => Symbol::ERR,
        }
    }
}

impl From<ProcessState> for Value {
    fn from(state: ProcessState) -> Self {
        Value::from(<&'static Symbol>::from(state))
    }
}

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
}

impl From<BuiltinProcessRef> for AnyProcessRef {
    fn from(builtin_process: BuiltinProcessRef) -> Self {
        Self(Value::from(builtin_process))
    }
}

impl From<UserProcessRef> for AnyProcessRef {
    fn from(user_process: UserProcessRef) -> Self {
        Self(Value::from(user_process))
    }
}

impl ProcessRef for AnyProcessRef {
    fn state(&self) -> ProcessState {
        match self.builtin_or_user_defined() {
            Left(proc) => proc.state(),
            Right(proc) => proc.state(),
        }
    }

    fn state_mut(&mut self) -> &mut ProcessState {
        match self.builtin_or_user_defined() {
            Left(mut builtin) => {
                let ref_mut = builtin.state_mut();
                unsafe { mem::transmute::<&mut ProcessState, &mut ProcessState>(ref_mut) }
            }
            Right(mut user_defined) => {
                let ref_mut = user_defined.state_mut();
                unsafe { mem::transmute::<&mut ProcessState, &mut ProcessState>(ref_mut) }
            }
        }
    }

    fn output_slot(&self) -> Value {
        match self.builtin_or_user_defined() {
            Left(builtin) => builtin.output_slot(),
            Right(user_defined) => user_defined.output_slot(),
        }
    }

    fn output_slot_mut(&mut self) -> &mut Value {
        match self.builtin_or_user_defined() {
            Left(mut builtin) => {
                let ref_mut = builtin.output_slot_mut();
                unsafe { mem::transmute::<&mut Value, &mut Value>(ref_mut) }
            }
            Right(mut user_defined) => {
                let ref_mut = user_defined.output_slot_mut();
                unsafe { mem::transmute::<&mut Value, &mut Value>(ref_mut) }
            }
        }
    }
}
