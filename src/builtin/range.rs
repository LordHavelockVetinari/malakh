use std::cmp::Ordering;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

struct NanError;

enum Command {
    Start,
    End,
    Step,
    None,
}

pub struct Range {
    start: Value,
    end: Option<Value>,
    step: Option<Value>,
    is_inclusive: bool,
    is_reversed: bool,
    command: Command,
}

impl Range {
    fn next(&mut self, vm: &mut Vm) -> Result<Option<Value>, NanError> {
        let start = self.start;
        let end = self.end.unwrap();
        let step = self.step.unwrap();
        match start.compare(end).unwrap() {
            Some(Ordering::Less) if self.is_reversed => Ok(None),
            Some(Ordering::Greater) if !self.is_reversed => Ok(None),
            Some(Ordering::Equal) if !self.is_inclusive => Ok(None),
            Some(_) => {
                self.start = start.add(step, vm.gc_mut()).unwrap();
                Ok(Some(start))
            }
            None => Err(NanError),
        }
    }
}

impl Function for Range {
    const NAME: &str = "Range";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Range {
            start: Value::ZERO,
            end: None,
            step: None,
            is_inclusive: false,
            is_reversed: false,
            command: Command::Start,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.start);
        if let Some(end) = self.end {
            gc.mark(end);
        }
        if let Some(step) = self.step {
            gc.mark(step);
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        match self.command {
            Command::Start => {
                if !input.is_number() {
                    err!(
                        vm,
                        "type error: {} start is {}",
                        Self::NAME,
                        input.type_name()
                    )
                }
                self.start = input;
                self.command = Command::None;
                Action::OptionalInput
            }
            Command::None => {
                let Some(cmd) = input.as_symbol() else {
                    err!(
                        vm,
                        "type error: {} expected method; got {}",
                        Self::NAME,
                        input.type_name()
                    );
                };
                if cmd == Symbol::TO {
                    self.command = Command::End;
                } else if cmd == Symbol::THROUGH {
                    self.is_inclusive = true;
                    self.command = Command::End;
                } else if cmd == Symbol::STEP {
                    self.command = Command::Step;
                } else {
                    err!(vm, "undefined method: <range> {:?}", input);
                }
                Action::Input
            }
            Command::End => {
                if !input.is_number() {
                    err!(
                        vm,
                        "type error: {} end is {}",
                        Self::NAME,
                        input.type_name()
                    )
                }
                if self.end.is_some() {
                    err!(vm, "{} got end twice", Self::NAME);
                }
                self.end = Some(input);
                self.command = Command::None;
                Action::OptionalInput
            }
            Command::Step => {
                if !input.is_number() {
                    err!(
                        vm,
                        "type error: {} step is {}",
                        Self::NAME,
                        input.type_name()
                    )
                }
                if self.step.is_some() {
                    err!(vm, "{} got step twice", Self::NAME);
                }
                self.step = Some(input);
                if input.compare(Value::ZERO).unwrap() == Some(Ordering::Less) {
                    self.is_reversed = true;
                }
                self.command = Command::None;
                Action::OptionalInput
            }
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        if self.end.is_none() {
            self.end = Some(self.start);
            self.start = Value::ZERO;
        }
        if self.step.is_none() {
            self.step = Some(Value::ONE);
        }
        match self.next(vm) {
            Err(NanError) => err!(vm, "{} got NaN", Self::NAME),
            Ok(Some(value)) => Action::Output(value),
            Ok(None) => Action::Stop,
        }
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        match self.next(vm) {
            Err(NanError) => err!(vm, "{} got NaN", Self::NAME),
            Ok(Some(value)) => Action::Output(value),
            Ok(None) => Action::Stop,
        }
    }
}
