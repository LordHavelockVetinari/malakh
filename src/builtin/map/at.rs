use crate::builtin::helper::{Action, Method};
use crate::builtin::map::{AbsentEntry, HashableValue};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

#[allow(clippy::enum_variant_names)]
enum Command {
    At,
    ChooseCommand,
    Set,
    Is,
}

pub struct At {
    key: Option<Value>,
    command: Command,
}

impl Method for At {
    type Parent = super::Map;

    fn new(_parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action) {
        let key = vm.try_take_temporary1();
        let (command, action) = if key.is_some() {
            (Command::ChooseCommand, Action::OptionalInput)
        } else {
            (Command::At, Action::Input)
        };
        (Self { key, command }, action)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(key) = self.key {
            gc.mark(key);
        }
    }

    fn input(&mut self, input: Value, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        match self.command {
            Command::At => {
                self.key = Some(input);
                self.command = Command::ChooseCommand;
                Action::OptionalInput
            }
            Command::ChooseCommand => {
                let Some(input) = input.as_symbol() else {
                    todo!("at didn't get a symbol");
                };
                if input == Symbol::SET {
                    self.command = Command::Set;
                    Action::Input
                } else if input == Symbol::IS {
                    self.command = Command::Is;
                    Action::Input
                } else if input == Symbol::REMOVE {
                    let key = self.key.expect("key should be initialized");
                    let Some(key) = HashableValue::new(key) else {
                        todo!("non-hashable key");
                    };
                    match parent.find_entry(key) {
                        Ok(occupied) => {
                            let value = occupied.get().1;
                            occupied.remove();
                            Action::Output(value)
                        }
                        Err(AbsentEntry { .. }) => todo!("remove nonexistent item"),
                    }
                } else {
                    todo!("index got unexpected symbol");
                }
            }
            Command::Set => {
                let key = self.key.expect("key should be initialized");
                let Some(key) = HashableValue::new(key) else {
                    todo!("non-hashable key");
                };
                // Use old key if replacing old entry.
                parent
                    .entry(key)
                    .and_modify(|entry| entry.1 = input)
                    .or_insert((key, input));
                Action::Stop
            }
            Command::Is => {
                let key = self.key.expect("key should be initialized");
                let Some(key) = HashableValue::new(key) else {
                    todo!("non-hashable key");
                };
                parent
                    .entry(key)
                    .and_modify(|_| todo!("`Is` got existing key"))
                    .or_insert((key, input));
                Action::Stop
            }
        }
    }

    fn no_input(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        let key = self.key.expect("key should be initialized");
        let Some(key) = HashableValue::new(key) else {
            return Action::Stop;
        };
        if let Ok(entry) = parent.find_entry(key) {
            Action::Output(entry.get().1)
        } else {
            Action::Stop
        }
    }
}
