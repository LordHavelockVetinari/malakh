use crate::builtin::helper::{Action, Method, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

enum Command {
    Set,
    Insert,
}

pub struct Index {
    index: Value,
    command: Option<Command>,
}

impl Method for Index {
    type Parent = super::List;

    fn new(_parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action) {
        let index = vm.take_temporary1();
        debug_assert!(index.is_int());
        let this = Self {
            index,
            command: None,
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.index);
    }

    fn input(&mut self, input: Value, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        match self.command {
            None => {
                let Some(input) = input.as_symbol() else {
                    err!(vm, "type error: <list> <index> {}", input.type_name());
                };
                if input == Symbol::SET {
                    self.command = Some(Command::Set);
                    Action::Input
                } else if input == Symbol::INSERT {
                    self.command = Some(Command::Insert);
                    Action::Input
                } else if input == Symbol::REMOVE {
                    if let Some(index) = self.index.as_usize()
                        && index < parent.data.len()
                    {
                        let value = parent.data.remove(index);
                        Action::Output(value)
                    } else {
                        err!(vm, "list index out of bounds");
                    }
                } else {
                    err!(vm, "undefined method: <list> <index> .{}", input.name());
                }
            }
            Some(Command::Set) => {
                if let Some(index) = self.index.as_usize()
                    && index < parent.data.len()
                {
                    parent.data[index] = input;
                } else {
                    err!(vm, "list index out of bounds");
                }
                Action::Stop
            }
            Some(Command::Insert) => {
                if let Some(index) = self.index.as_usize()
                    && index <= parent.data.len()
                {
                    parent.data.insert(index, input);
                } else {
                    err!(vm, "list index out of bounds");
                }
                Action::Stop
            }
        }
    }

    fn no_input(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if let Some(index) = self.index.as_usize()
            && let Some(&value) = parent.data.get(index)
        {
            Action::Output(value)
        } else {
            Action::Stop
        }
    }
}
