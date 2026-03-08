use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

#[derive(PartialEq, Eq)]
enum Command {
    String,
    Keyword,
    Start,
    End,
}

pub struct Slice {
    string: Option<Value>,
    start: Option<usize>,
    end: Option<usize>,
    is_inclusive: bool,
    command: Command,
}

impl Function for Slice {
    const NAME: &str = "Slice";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Slice {
            string: None,
            start: None,
            end: None,
            is_inclusive: false,
            command: Command::String,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(s) = self.string {
            gc.mark(s);
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        match self.command {
            Command::String => {
                if !input.is_string() {
                    err!(vm, "type error: {} {}", Self::NAME, input.type_name());
                }
                self.string = Some(input);
                self.command = Command::Keyword;
                Action::Input
            }
            Command::Keyword => {
                let Some(input) = input.as_symbol() else {
                    err!(
                        vm,
                        "type error: Slice expected a method; got {}",
                        input.type_name()
                    );
                };
                if input == Symbol::FROM {
                    if self.start.is_some() {
                        err!(vm, "Slice got start twice");
                    }
                    self.command = Command::Start;
                } else if input == Symbol::TO {
                    if self.end.is_some() {
                        err!(vm, "Slice got end twice");
                    }
                    self.command = Command::End;
                } else if input == Symbol::THROUGH {
                    if self.end.is_some() {
                        err!(vm, "Slice got end twice");
                    }
                    self.is_inclusive = true;
                    self.command = Command::End;
                } else {
                    err!(vm, "undefined method: <slice> .{}", input.name());
                }
                Action::Input
            }
            Command::Start => {
                let Some(n) = input.as_usize_saturating() else {
                    err!(vm, "type error: Slice start is {}", input.type_name());
                };
                self.start = Some(n);
                self.command = Command::Keyword;
                Action::OptionalInput
            }
            Command::End => {
                let Some(n) = input.as_usize_saturating() else {
                    err!(vm, "type error: Slice end is {}", input.type_name());
                };
                self.end = Some(n);
                self.command = Command::Keyword;
                Action::OptionalInput
            }
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        assert!(self.command == Command::Keyword);
        if self.start.is_none() && self.end.is_none() {
            err!(vm, "Slice must get either start or end");
        }
        let s = self
            .string
            .expect("slice should have gotten a string at this point")
            .as_string_ref()
            .expect("slice should check its string input");
        let bytes = s.bytes();
        let start = self.start.unwrap_or(0).min(bytes.len());
        let end = self
            .end
            .unwrap_or(usize::MAX)
            .saturating_add(self.is_inclusive as usize)
            .min(bytes.len())
            .max(start);
        let slice = s
            .slice(start, end - start, vm.gc_mut())
            .expect("slicing shouldn't fail");
        Action::Output(Value::from(slice))
    }
}
