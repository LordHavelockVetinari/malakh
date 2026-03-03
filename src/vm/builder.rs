use malachite::Integer;

use crate::builtin;
use crate::vm::gc::GarbageCollector;
use crate::vm::global_variable::GlobalVariable;
use crate::vm::macros::leak_code;
use crate::vm::options::VmOptions;
use crate::vm::string::StringRef;
use crate::vm::symbol::Symbol;
use crate::vm::user_process::UserProcessFamily;
use crate::vm::{Value, Vm};

pub struct VmBuilderInner {
    constants: Vec<Value>,
    process_families: Vec<&'static UserProcessFamily>,
    global_variable_families: Vec<Option<u32>>,
    // The initial process family is the first process that runs.
    // It can't capture anything, and can't stop or pause.
    initial_process_family: Option<u32>,
    gc: GarbageCollector,
}

pub struct VmBuilder(Box<VmBuilderInner>);

impl VmBuilder {
    pub fn new() -> Self {
        Self(Box::new(VmBuilderInner {
            constants: Vec::new(),
            process_families: Vec::new(),
            global_variable_families: Vec::new(),
            initial_process_family: None,
            gc: GarbageCollector::new(),
        }))
    }

    pub fn constant(&mut self, value: Value) -> Option<u32> {
        let index = u32::try_from(self.0.constants.len()).ok()?;
        self.0.constants.push(value);
        Some(index)
    }

    pub fn int_constant(&mut self, value: Integer) -> Option<u32> {
        let value = Value::alloc_from(value, &mut self.0.gc);
        self.constant(value)
    }

    pub fn float_constant(&mut self, value: f64) -> Option<u32> {
        let value = Value::alloc_from(value, &mut self.0.gc);
        self.constant(value)
    }

    pub fn string_symbol(&mut self, s: &[u8]) -> Option<u32> {
        let value = Value::from(StringRef::new(s, &mut self.0.gc));
        self.constant(value)
    }

    pub fn symbol_constant(&mut self, name: &str) -> Option<u32> {
        self.constant(Value::from(Symbol::get_global(name)))
    }

    pub fn process_family(&mut self, family: UserProcessFamily) -> Option<u32> {
        let family = Box::leak(Box::new(family));
        let index = u32::try_from(self.0.process_families.len()).ok()?;
        self.0.process_families.push(family);
        Some(index)
    }

    pub fn initial_process_family(&mut self, family: UserProcessFamily) -> Option<u32> {
        let index = self.process_family(family)?;
        self.0.initial_process_family = Some(index);
        Some(index)
    }

    #[must_use]
    pub fn global_variable(&mut self, family: Option<u32>) -> Option<u32> {
        let index = u32::try_from(self.0.global_variable_families.len()).ok()?;
        self.0.global_variable_families.push(family);
        Some(index)
    }

    pub fn build(self) -> Vm {
        let Some(initial_family) = self.0.initial_process_family else {
            panic!("VM constructed without initial process");
        };
        let globals: Vec<&'static GlobalVariable> = self
            .0
            .global_variable_families
            .into_iter()
            .map(|family| {
                let family = family.map(|f| self.0.process_families[f as usize]);
                &*Box::leak(Box::new(GlobalVariable::new(family)))
            })
            .collect();
        let init_instructions = leak_code! {
            NEW 0, initial_family;
            UNREACHABLE 0, 0, 0;
        }
        .as_ptr();
        for family in &self.0.process_families {
            assert!(
                family
                    .try_bodies
                    .iter()
                    .is_sorted_by_key(|body| body.end.addr() - body.start.addr())
            );
        }
        Vm {
            constants: self.0.constants,
            user_process_families: self.0.process_families,
            builtin_process_families: *builtin::BUILTINS,
            global_variables: globals,
            call_stack: Vec::new(),
            instruction_pointer: init_instructions,
            memory: vec![Value::default()].leak(),
            gc: self.0.gc,
            temporary1: None,
            options: VmOptions::default(),
        }
    }
}
