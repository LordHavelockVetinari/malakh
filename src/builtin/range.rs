use std::cmp::Ordering;

use crate::builtin::helper::{self, Action};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

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
    fn next(&mut self, vm: &mut Vm) -> Option<Value> {
        let start = self.start;
        let end = self.end.unwrap();
        let step = self.step.unwrap();
        match start.compare(end).unwrap() {
            Some(Ordering::Less) if self.is_reversed => None,
            Some(Ordering::Greater) if !self.is_reversed => None,
            Some(Ordering::Equal) if !self.is_inclusive => None,
            Some(_) => {
                self.start = start.add(step, vm.gc_mut()).unwrap();
                Some(start)
            }
            None => {
                todo!("Range got NaN")
            }
        }
    }
}

impl helper::Function for Range {
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

    fn input(&mut self, input: Value, _vm: &mut Vm) -> Action {
        match self.command {
            Command::Start => {
                if !input.is_number() {
                    todo!("range didn't get a number")
                }
                self.start = input;
                self.command = Command::None;
                Action::OptionalInput
            }
            Command::None => {
                let Some(input) = input.as_symbol() else {
                    todo!("Range expected a symbol");
                };
                if input == Symbol::TO {
                    self.command = Command::End;
                } else if input == Symbol::THROUGH {
                    self.is_inclusive = true;
                    self.command = Command::End;
                } else if input == Symbol::STEP {
                    self.command = Command::Step;
                } else {
                    todo!("bad argument to range");
                }
                Action::Input
            }
            Command::End => {
                if !input.is_number() {
                    todo!("range didn't get a number")
                }
                if self.end.is_some() {
                    todo!("double ending in range");
                }
                self.end = Some(input);
                self.command = Command::None;
                Action::OptionalInput
            }
            Command::Step => {
                if !input.is_number() {
                    todo!("range didn't get a number")
                }
                if self.step.is_some() {
                    todo!("double step in range");
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
            Some(value) => Action::Output(value),
            None => Action::Stop,
        }
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        match self.next(vm) {
            Some(value) => Action::Output(value),
            None => Action::Stop,
        }
    }
}
