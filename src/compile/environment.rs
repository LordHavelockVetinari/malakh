use std::collections::HashMap;
use std::rc::Rc;

use either::Either::{self, Left, Right};

use crate::compile::error::CompilationError;
use crate::compile::register_allocator::RegisterAllocator;
use crate::parse::location::Location;
use crate::parse::tree::{Expr, ImportDeclaration};

#[derive(Clone, Debug)]
pub enum GlobalDefinition {
    Variable {
        global_index: u32,
    },
    Constructor {
        generator_family: u32,
        result_expr: Rc<Expr>,
    },
    BuiltinConstructor {
        index: u32,
    },
    BuiltinConstant {
        index: u32,
    },
    Import {
        decl: Rc<ImportDeclaration>,
        index: usize,
    },
}

#[derive(Clone)]
pub enum LocalDefinition {
    Variable {
        index: u16,
    },
    CapturedVariable {
        index: u16,
    },
    #[allow(unused)]
    Constructor {
        process_family: u32,
    },
}

pub struct Scope {
    definition_names: Vec<String>,
}

pub struct ProcessEnvironment {
    global_names: Option<Rc<HashMap<String, GlobalDefinition>>>,
    local_names: HashMap<String, LocalDefinition>,
    scopes: Vec<Scope>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            definition_names: Vec::new(),
        }
    }
}

impl ProcessEnvironment {
    pub fn new() -> Self {
        Self {
            global_names: None,
            local_names: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn init_global_names(&mut self, global_names: Rc<HashMap<String, GlobalDefinition>>) {
        if self.global_names.is_some() {
            panic!("attempt to initialize global_names twice");
        }
        self.global_names = Some(global_names);
    }

    fn global_names(&self) -> &HashMap<String, GlobalDefinition> {
        self.global_names
            .as_ref()
            .expect("global_names uninitialized")
    }

    pub fn get_definition(
        &self,
        name: &str,
    ) -> Option<Either<&GlobalDefinition, &LocalDefinition>> {
        if let Some(def) = self.global_names().get(name) {
            Some(Left(def))
        } else if let Some(def) = self.local_names.get(name) {
            Some(Right(def))
        } else {
            None
        }
    }

    pub fn enter_new_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit_scope(&mut self, alloc: &mut RegisterAllocator) {
        let scope = self
            .scopes
            .pop()
            .expect("cannot call exit_scope outside of any scope");
        for name in scope.definition_names {
            let def = self
                .local_names
                .remove(&name)
                .expect("name should be in definition map");
            match def {
                LocalDefinition::Constructor { .. } => {}
                LocalDefinition::Variable { index }
                | LocalDefinition::CapturedVariable { index } => {
                    alloc.dealloc(index);
                }
            }
        }
    }

    pub fn add_local(
        &mut self,
        name: String,
        definition: LocalDefinition,
        location: &Location,
    ) -> Result<(), CompilationError> {
        if self.get_definition(&name).is_some() {
            return CompilationError::err(format!("redefinition of name `{}`", name), location);
        }
        self.scopes
            .last_mut()
            .expect("add_local can only be called in a scope")
            .definition_names
            .push(name.clone());
        self.local_names.insert(name, definition);
        Ok(())
    }
}
